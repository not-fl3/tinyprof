use frontends::ProfilerFrontend;
use counters::{FrameReport, ReportCounter};

use std::collections::HashMap;

use gag::BufferRedirect;
use termion;
use termion::raw::{IntoRawMode, RawTerminal};
use std::io::{stdin, stdout, Read, Stdout, Write};
use termion::input::TermRead;

use std::sync::mpsc::*;

pub struct TermionBuildParams {
    /// Width of profiler area.
    /// Will be half of the screen instead.
    pub profiler_width: Option<u16>,

    /// Width of logger area.
    /// Will be (width of the screen - profiler_width) instead.
    /// Setting both profiler and logger width may be a workaround for "Inappropriate ioctl for device" error.
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
    current_line: u16,
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
        for _ in 0 .. n {
            write!(self.stdout, "\n").unwrap();
        }
    }
    pub fn put_string(&mut self, string: &str) {
        if self.current_line >= self.height - 2 {
            return;
        }

        let string = TerminalHelper::append_spaces(string, self.width as usize);
        write!(
            self.stdout,
            "{}",
            termion::cursor::Left(self.terminal_width)
        ).unwrap();
        write!(self.stdout, "{}", termion::cursor::Right(self.x)).unwrap();
        write!(self.stdout, "{}", string).unwrap();
        write!(self.stdout, "\n").unwrap();
        self.current_line += 1;
    }

    pub fn fill_to_end(&mut self) {
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

    fn append_spaces(string: &str, count: usize) -> String {
        let mut res = String::new();
        res.push('║');
        for c in string.chars().take(count) {
            res.push(c);
        }

        if res.len() <= count {
            for _ in res.len()..count {
                res.push(' ');
            }
        }
        res.push('║');
        assert_eq!(res.len(), count + 3);
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

        // Profiler
        self.terminal
            .text_area(0, 0, self.profiler_width, self.profiler_height);

        self.terminal
            .put_string("PROFILER, Arrows - navigate, space - collapse, Q - exit:");

        for thread in self.thread_data.values() {
            draw_frame_report(&mut self.terminal, thread);
        }
        self.terminal.fill_to_end();

        // Variables
        self.terminal.text_area(
            0,
            self.profiler_height,
            self.profiler_width,
            self.variables_height,
        );

        self.terminal.put_string("VARIABLES:");

        self.terminal.fill_to_end();

        // Stdout
        self.terminal.text_area(
            self.profiler_width,
            0,
            self.logger_width,
            self.terminal_height,
        );
        self.terminal.put_string("STDOUT: ");

        for line in self.logs.split('\n').rev().take(self.params.logs_count) {
            self.terminal.put_string(line);
        }
        self.terminal.fill_to_end();

        for c in self.char_receiver.try_recv() {
            if let termion::event::Key::Char('q') = c {
                writeln!(self.terminal.stdout, "{}", termion::cursor::Show).unwrap();
                return false;
            }
        }

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
    terminal.put_string(&frame.thread_name);
    for counter in &frame.counters {
        draw_counter_recursive(terminal, counter, 4);
    }
}

fn draw_counter_recursive(
    terminal: &mut TerminalHelper,
    counter: &ReportCounter,
    shift: u16,
) -> u16 {
    let mut lines = 1;

    terminal.put_string(&format!(
        "{}{}: {:.3}ms",
        " ".repeat(shift as usize),
        counter.name,
        counter.duration
    ));

    for counter in &counter.counters {
        lines += draw_counter_recursive(terminal, counter, shift + 4);
    }
    lines
}
