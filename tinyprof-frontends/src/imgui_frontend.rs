use ProfilerFrontend;
use tinyprof::{FrameReport, ReportCounter};

use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;
use lazysort::SortedBy;
use imgui::*;

/// Imgui frontend options
pub struct ImguiBuildParams {
    /// Imgui frontend stores that amout of frames per each thread
    pub frames_history: usize,

    /// Default is true - frames will be recorded after frontent initialization
    /// May be usefull for extremely long frames_history and memory problems.
    pub record_on_start: bool,

    pub history_area_width: i32,
    pub history_area_height: i32,
}

impl Default for ImguiBuildParams {
    fn default() -> ImguiBuildParams {
        ImguiBuildParams {
            frames_history: 666,
            record_on_start: true,
            history_area_width: 500,
            history_area_height: 30,
        }
    }
}

#[derive(Clone)]
struct ImguiFrameReport {
    duration: f64,
    frame: FrameReport,
}
struct ThreadData {
    reports: VecDeque<ImguiFrameReport>,
}

impl ThreadData {
    fn new() -> ThreadData {
        ThreadData {
            reports: VecDeque::new(),
        }
    }
}

pub struct ImguiFrontend {
    threads_data: HashMap<String, ThreadData>,
    frames_history: usize,
    selected_frame: Option<ImguiFrameReport>,
    pause: bool,
    history_area_width: f32,
    history_area_height: f32,
}

impl ImguiFrontend {
    pub fn new(params: ImguiBuildParams) -> ImguiFrontend {
        ImguiFrontend {
            threads_data: HashMap::new(),
            frames_history: params.frames_history,
            pause: params.record_on_start == false,
            selected_frame: None,
            history_area_width: params.history_area_width as f32,
            history_area_height: params.history_area_height as f32
        }
    }

    fn truncate(&mut self) {
        for (_, thread_data) in &mut self.threads_data {
            thread_data.reports.truncate(self.frames_history);
        }
    }
    pub fn draw<'a>(&mut self, ui: &Ui<'a>) {
        ui.window(im_str!("Profiler"))
            .flags(ImGuiWindowFlags_AlwaysAutoResize)
            .resizable(true)
            .movable(true)
            .build(|| {
                if self.pause == false {
                    if ui.small_button(im_str!("pause")) {
                        self.pause = true;
                    }
                } else {
                    if ui.small_button(im_str!("unpause")) {
                        self.selected_frame = None;
                        self.pause = false;
                    }
                }

                for (thread_name, thread_data) in &self.threads_data {
                    if let Some(frame) = draw_history_graph(
                        ui,
                        (self.history_area_width, self.history_area_height),
                        &thread_name,
                        &thread_data,
                    ) {
                        self.selected_frame = Some(thread_data.reports[frame].clone());
                        self.pause = true;
                    }
                    let data = thread_data.reports.front();
                    if let Some(data) = data {
                        draw_report(&data, ui);
                    }
                    ui.separator();
                }
            });

        ui.window(im_str!("Variables"))
            .flags(ImGuiWindowFlags_AlwaysAutoResize)
            .resizable(true)
            .movable(true)
            .build(|| {
                for (thread_name, thread_data) in &self.threads_data {
                    let data = self.selected_frame.as_ref().or(thread_data.reports.front());

                    if let Some(data) = data {
                        ui.tree_node(im_str!("{}", thread_name)).build(|| {
                            for variable in data.frame.variables.iter() {
                                ui.text(im_str!("{}: {}", variable.0, variable.1));
                            }
                        });
                    }
                }
            });

        self.truncate();
    }
}

fn draw_history_graph<'a>(
    ui: &Ui<'a>,
    (w, h): (f32, f32),
    name: &str,
    frame: &ThreadData,
) -> Option<usize> {
    let mut selected_report = None;

    ui.with_child(
        im_str!("{}", name),
        ImVec2::new(w, h),
        false,
        ImGuiWindowFlags_NoTitleBar,
        || {
            let mut draw_list = ui.get_window_draw_list();
            let cursor_pos = igGetCursorPos();
            let win_pos = igGetCursorScreenPos();
            let canvas_sz = igGetWindowSize();
            let mouse_pos = ui.imgui().mouse_pos();
            let draw_pos = win_pos + cursor_pos;
            let selected_x = mouse_pos.0 - win_pos.x;
            let mut highlighted_report = None;

            if mouse_pos.0 >= win_pos.x && mouse_pos.0 <= win_pos.x + canvas_sz.x
                && mouse_pos.1 <= win_pos.y + canvas_sz.y && mouse_pos.1 >= win_pos.y
            {
                let selected = (selected_x / canvas_sz.x * frame.reports.len() as f32) as usize;
                if igIsMouseClicked(0) {
                    selected_report = Some(selected);
                }
                highlighted_report = Some(selected);
            }

            let max_time = frame.reports.iter().fold(0.0, |max, c| c.duration.max(max));

            for i in 0..frame.reports.len() {
                let h = frame.reports[i].duration as f32 / max_time as f32 * canvas_sz.y;
                let x = i as f32 / frame.reports.len() as f32 * canvas_sz.x;

                let color = match highlighted_report {
                    Some(selected) if selected == i => 0xffffffff,
                    _ => 0xaaaaaaff,
                };

                draw_list.add_line(
                    ImVec2::new(draw_pos.x + x, draw_pos.y + canvas_sz.y),
                    ImVec2::new(draw_pos.x + x, draw_pos.y + canvas_sz.y - h),
                    color,
                    1.0,
                );
            }
        },
    );
    selected_report
}

impl ProfilerFrontend for ImguiFrontend {
    fn receive_reports(&mut self, report: FrameReport) {
        if self.pause {
            return;
        }

        let duration = report
            .counters
            .iter()
            .fold(0.0, |res, c| res + c.duration.unwrap_or(0.0));

        let thread_data = self.threads_data
            .entry(report.thread_name.clone())
            .or_insert(ThreadData::new());

        thread_data.reports.push_front(ImguiFrameReport {
            duration: duration,
            frame: report.clone(),
        });
    }
}

fn draw_report<'a>(frame: &ImguiFrameReport, ui: &Ui<'a>) {
    ui.tree_node(im_str!("{}", frame.frame.thread_name))
        .build(|| {
            for counter in frame.frame.counters.iter().sorted_by(|a, b| {
                b.duration
                    .partial_cmp(&a.duration)
                    .unwrap_or(Ordering::Less)
            }) {
                draw_counter_recursive(counter, ui, frame.duration);
            }
        });
}

fn draw_counter_recursive<'a>(counter: &ReportCounter, ui: &Ui<'a>, duration: f64) {
    let label = match counter.duration {
        Some(counter_duration) => im_str!(
            "{}: {:.4}ms ({:.0}fps) {:.0}%",
            counter.name,
            counter_duration,
            1.0 / counter_duration,
            counter_duration / duration * 100.0
        ),
        None => im_str!("{}: NOT FINISHED", counter.name),
    };

    ui.tree_node(im_str!("{}", counter.id))
        .label(label)
        .build(|| {
            let duration = counter.duration;

            for counter in counter.counters.iter().sorted_by(|a, b| {
                b.duration
                    .partial_cmp(&a.duration)
                    .unwrap_or(Ordering::Less)
            }) {
                draw_counter_recursive(counter, ui, duration.unwrap_or(::std::f64::INFINITY));
            }
        });
}
