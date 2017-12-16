use ProfilerFrontend;
use tinyprof::{FrameReport, ReportCounter};

use std::collections::{HashMap, HashSet};

use gag::BufferRedirect;
use termion;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{stdin, stdout, Read, Stdout, Write};
use termion::input::TermRead;

use std::sync::mpsc::*;

/// Termion frontend options
pub struct TermionBuildParams {
    /// Width of profiler area.
    /// Will be half of the screen instead.
    pub profiler_width: Option<u16>,

    /// Width of logger area.
    /// Will be (width of the screen - profiler_width) instead.
    /// Setting both profiler and logger width may be a workaround
    /// for "Inappropriate ioctl for device" error.
    pub logger_width: Option<u16>,

    /// Height of entire output area.
    /// By default will take 2/3 of available space.
    pub height: Option<u16>,

    /// Height of variables area.
    /// Will be 1/3 of the screen instead.
    /// Profiler area will be ```height - variables_height```.
    pub variables_height: Option<u16>,

    /// Amount of log strings displayed on the right
    pub logs_count: usize,
}

impl Default for TermionBuildParams {
    fn default() -> Self {
        TermionBuildParams {
            profiler_width: None,
            logger_width: None,
            height: None,
            variables_height: None,
            logs_count: 40,
        }
    }
}

struct TerminalHelper {
    pub stdout: RawTerminal<Stdout>,

    x: u16,
    y: u16,
    width: u16,
    height: u16,
    terminal_width: u16,
    terminal_height: u16,
    current_line: u16,
    cursor: u16,
    active_window: u16,
    current_window: u16,
    space_pressed: bool,
    collapsed_lines: HashSet<String>,
}

impl TerminalHelper {
    pub fn new(stdout: RawTerminal<Stdout>, width: u16, height: u16) -> TerminalHelper {
        TerminalHelper {
            stdout: stdout,
            current_line: 0,
            x: 0,
            y: 0,
            width: width,
            height: height,
            terminal_width: width,
            terminal_height: height,
            cursor: 0,
            active_window: 0,
            current_window: 0,
            space_pressed: false,
            collapsed_lines: HashSet::new(),
        }
    }

    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor < self.terminal_height - 1 {
            self.cursor += 1;
        }
    }

    pub fn text_area(&mut self, x: u16, y: u16, width: u16, height: u16) {
        write!(
            self.stdout,
            "{}",
            termion::cursor::Up(self.current_line + self.y)
        ).unwrap();

        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
        self.down(y);

        self.current_line = 1;

        write!(
            self.stdout,
            "{}",
            termion::cursor::Left(self.terminal_width)
        ).unwrap();
        write!(self.stdout, "{}", termion::cursor::Right(self.x)).unwrap();
        write!(
            self.stdout,
            "╔{}╗\n",
            "═".repeat(self.width as usize - 3)
        ).unwrap();
    }

    fn down(&mut self, n: u16) {
        for _ in 0..n {
            write!(self.stdout, "\n").unwrap();
        }
    }

    pub fn put_string(&mut self, shift: u16, string: &str, id: Option<String>) -> bool {
        if self.current_line >= self.height - 2 {
            return false;
        }

        let id = id.unwrap_or(string.to_string());
        let collapsed = self.collapsed_lines.contains(&id);
        let string = TerminalHelper::append_spaces(
            &format!(
                "{}{}{}",
                " ".repeat(if shift >= 1 { shift - 1 } else { shift } as usize),
                if collapsed { "+" } else { " " },
                string
            ),
            self.width as usize - 3,
        );

        write!(
            self.stdout,
            "{}",
            termion::cursor::Left(self.terminal_width)
        ).unwrap();
        write!(self.stdout, "{}", termion::cursor::Right(self.x)).unwrap();
        if self.current_window == self.active_window && self.current_line == self.cursor {
            write!(
                self.stdout,
                "║{}{}{}║",
                termion::color::Bg(termion::color::Green),
                string,
                termion::color::Bg(termion::color::Reset)
            ).unwrap();
        } else {
            write!(self.stdout, "║{}║", string).unwrap();
        }
        write!(self.stdout, "\n").unwrap();

        if self.line_pressed(self.current_line) && self.collapsed_lines.contains(&id) {
            self.collapsed_lines.remove(&id);
        } else if self.line_pressed(self.current_line)
            && self.collapsed_lines.contains(&id) == false
        {
            self.collapsed_lines.insert(id.clone());
        }

        self.current_line += 1;

        return collapsed == false;
    }

    pub fn next_frame(&mut self) {
        self.current_window = 0;
        self.space_pressed = false;
    }

    pub fn end_text_area(&mut self) {
        self.current_window += 1;

        for _ in self.current_line..self.height - 1 {
            write!(
                self.stdout,
                "{}",
                termion::cursor::Left(self.terminal_width)
            ).unwrap();
            write!(self.stdout, "{}", termion::cursor::Right(self.x)).unwrap();
            write!(
                self.stdout,
                "║{}║\n",
                " ".repeat(self.width as usize - 3)
            ).unwrap();
        }
        write!(
            self.stdout,
            "{}",
            termion::cursor::Left(self.terminal_width)
        ).unwrap();
        write!(self.stdout, "{}", termion::cursor::Right(self.x)).unwrap();
        write!(
            self.stdout,
            "╚{}╝\n",
            "═".repeat(self.width as usize - 3)
        ).unwrap();

        self.current_line = self.height;
    }

    pub fn set_space_key(&mut self) {
        self.space_pressed = true;
    }

    fn line_pressed(&self, line: u16) -> bool {
        self.space_pressed && self.current_window == self.active_window && line == self.cursor
    }
    fn append_spaces(string: &str, count: usize) -> String {
        let mut res = String::new();
        for c in string.chars().take(count) {
            res.push(c);
        }

        if res.len() < count {
            for _ in res.len()..count {
                res.push(' ');
            }
        }
        assert_eq!(res.len(), count);
        res
    }
}

pub struct TermionFrontend {
    thread_data: HashMap<String, FrameReport>,
    params: TermionBuildParams,
    buf_redirect: Option<BufferRedirect>,
    logs: String,
    profiler_width: u16,
    profiler_height: u16,
    variables_height: u16,
    terminal_height: u16,
    logger_width: u16,
    char_receiver: Receiver<termion::event::Key>,
    terminal: TerminalHelper,
}

impl TermionFrontend {
    pub fn new(params: TermionBuildParams) -> TermionFrontend {
        let terminal_size = termion::terminal_size().unwrap();

        let profiler_width = params.profiler_width.unwrap_or(terminal_size.0 / 2);
        let height = params.height.unwrap_or(terminal_size.1 * 2 / 3);
        let variables_height = params.variables_height.unwrap_or(height / 3);
        let profiler_height = height - variables_height;

        let logger_width = params
            .logger_width
            .unwrap_or(terminal_size.0 - profiler_width);
        let mut stdout = stdout().into_raw_mode().unwrap();

        write!(stdout, "\n{}", termion::cursor::Hide).unwrap();
        let (tx, rx) = channel();
        ::std::thread::spawn(move || {
            for c in stdin().keys() {
                tx.send(c.unwrap()).unwrap();
            }
        });
        TermionFrontend {
            params: params,
            thread_data: HashMap::new(),
            terminal: TerminalHelper::new(stdout, terminal_size.0, terminal_size.1),
            buf_redirect: Some(BufferRedirect::stdout().unwrap()),
            logs: String::new(),
            profiler_width: profiler_width,
            profiler_height: profiler_height,
            terminal_height: height,
            logger_width: logger_width,
            variables_height: variables_height,
            char_receiver: rx,
        }
    }

    pub fn draw(&mut self) -> bool {
        // lock stdout to prevent unwanted messages
        let stdout = stdout();
        let _handle = stdout.lock();

        // grab data from redirected stdout and remove redirecting (by dropping buf_redirect)
        let buf_redirect = self.buf_redirect.take();
        let mut new_logs = String::new();
        buf_redirect.unwrap().read_to_string(&mut new_logs).unwrap();
        self.logs.push_str(&new_logs);

        // set input for this frame
        for c in self.char_receiver.try_recv() {
            if let termion::event::Key::Char('q') = c {
                writeln!(self.terminal.stdout, "{}", termion::cursor::Show).unwrap();
                return false;
            }
            if let termion::event::Key::Up = c {
                self.terminal.cursor_up();
            }
            if let termion::event::Key::Down = c {
                self.terminal.cursor_down();
            }
            if let termion::event::Key::Char(' ') = c {
                self.terminal.set_space_key();
            }
        }

        // Profiler
        self.terminal
            .text_area(0, 0, self.profiler_width, self.profiler_height);

        self.terminal.put_string(
            0,
            "PROFILER, Arrows - navigate, space - collapse, Q - exit:",
            None,
        );

        for thread in self.thread_data.values() {
            draw_frame_report(&mut self.terminal, thread);
        }
        self.terminal.end_text_area();

        // Variables
        self.terminal.text_area(
            0,
            self.profiler_height,
            self.profiler_width,
            self.variables_height,
        );

        self.terminal.put_string(0, "VARIABLES:", None);
        for thread in self.thread_data.values() {
            self.terminal.put_string(0, &thread.thread_name, None);
            for variable in thread.variables.iter() {
                self.terminal
                    .put_string(0, &format!("{}: {}", variable.0, variable.1), None);
            }
        }


        self.terminal.end_text_area();

        // Stdout
        self.terminal.text_area(
            self.profiler_width,
            0,
            self.logger_width,
            self.terminal_height,
        );
        self.terminal.put_string(0, "STDOUT: ", None);

        for line in self.logs.split('\n').rev().take(self.params.logs_count) {
            self.terminal.put_string(0, line, None);
        }
        self.terminal.end_text_area();

        self.terminal.next_frame();

        // restore stdout redirect
        self.buf_redirect = Some(BufferRedirect::stdout().unwrap());

        return true;
    }
}

impl ProfilerFrontend for TermionFrontend {
    fn receive_reports(&mut self, report: FrameReport) {
        let name = report.thread_name.clone();

        self.thread_data.insert(name, report);
    }
}

fn draw_frame_report(terminal: &mut TerminalHelper, frame: &FrameReport) {
    if terminal.put_string(0, &frame.thread_name, None) {
        for counter in &frame.counters {
            draw_counter_recursive(terminal, counter, 4);
        }
    }
}

fn draw_counter_recursive(
    terminal: &mut TerminalHelper,
    counter: &ReportCounter,
    shift: u16,
) -> u16 {
    let mut lines = 1;

    if terminal.put_string(
        shift,
        &format!("{}: {:.3}ms", counter.name, counter.duration.unwrap_or(0.0)),
        Some(counter.id.to_string()),
    ) {
        for counter in &counter.counters {
            lines += draw_counter_recursive(terminal, counter, shift + 4);
        }
    }
    lines
}
