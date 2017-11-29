use std::cell::RefCell;
use failure::Error;
use reporter::{Reporter, reporter_id};
use counters::{Counter, FrameReport};
use vec_map::VecMap;
use std::collections::HashMap;

use std::ptr;

thread_local! {
    static STORAGE: RefCell<ThreadStorage> = RefCell::new(ThreadStorage::new());
}

fn with_mut_storage<T: FnOnce(&mut ThreadStorage) -> R, R>(f: T) -> R {
    STORAGE.with(|storage| {
        let mut storage = storage.borrow_mut();
        f(&mut *storage)
    })
}

struct Report {
    counters: Vec<Counter>,
    active_counter: *mut Counter,
}

impl Report {
    fn clear(&mut self) {
        self.counters.clear();
        self.active_counter = ptr::null_mut();
    }

    fn start_region(&mut self, name: &str, mut reporter : Reporter) {
        let counters = if self.active_counter.is_null() {
            &mut self.counters
        } else {
            unsafe { &mut self.active_counter.as_mut().unwrap().counters }
        };

        reporter.start();
        counters.push(Counter {
            name: name.to_string(),
            time: None,
            reporter: reporter,
            counters: vec![],
            parent: self.active_counter,
        });
        self.active_counter = counters.last_mut().unwrap() as *mut _;
    }

    fn end_region(&mut self) {
        let counter = unsafe { self.active_counter.as_mut().unwrap() };

        counter.time = Some(counter.reporter.end());
        self.active_counter = counter.parent;
    }

    fn empty(&self) -> bool {
        self.counters.len() == 0
    }
}

struct ThreadStorage {
    thread_name: String,
    reports: VecMap<Report>,
    variables: HashMap<String, f32>
}

impl ThreadStorage {
    pub fn new() -> ThreadStorage {
        let mut reports = VecMap::new();

        reports.insert(reporter_id::SYSTEM_CLOCK, Report {
            counters: vec![],
            active_counter: ptr::null_mut(),
        });
        reports.insert(reporter_id::CUSTOM, Report {
            counters: vec![],
            active_counter: ptr::null_mut(),
        });

        ThreadStorage {
            thread_name: "unknown".to_string(),
            reports: reports,
            variables: HashMap::new()
        }
    }
}

/// Thread name is using as an unique thread id.
/// Its fine to have two threads with the same name, but their data will be considered as one thread data in that case.
pub fn set_thread_name(name: &str) {
    with_mut_storage(|storage| storage.thread_name = name.to_string());
}

/// Send frame data to profiler and start a new frame.
/// Will panic if profiler's internal frames buffer is overloaded.
/// Also it should panic(but not yet!) if called with any active regions.
pub fn next_frame() -> Result<(), Error> {
    with_mut_storage(|storage| {
        for report in storage.reports.iter_mut().filter(|r| r.1.empty() == false) {
            let thread_name = if report.0 == 0 {
                storage.thread_name.clone()
            } else {
                format!("{} {}", storage.thread_name, report.0)
            };
            let frame = FrameReport::from_thread_data(&thread_name, &report.1.counters);
            report.1.clear();

            if let Err(err) = ::profiler::send_profiler_state(frame) {
                return Err(err);
            }
        }
        return Ok(())
    })
}

pub fn set_variable_value(name: String, value: f32) {
    with_mut_storage(|storage| {
        storage.variables.insert(name, value)
    });
}

pub(crate) fn start_region(name: &str, reporter: Reporter, reporter_id : usize) {
    with_mut_storage(|storage| {
        storage.reports[reporter_id].start_region(name, reporter);
    });
}

pub(crate) fn end_region(reporter_id : usize) {
    with_mut_storage(|storage| {
        storage.reports[reporter_id].end_region();
        storage.variables.clear();
    });
}
