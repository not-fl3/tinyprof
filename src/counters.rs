use reporter::Reporter;

pub(crate) struct Counter {
    pub(crate) name: String,
    pub(crate) time: Option<f64>,
    pub(crate) reporter: Reporter,
    pub(crate) counters: Vec<Counter>,
    pub(crate) parent: *mut Counter,
}

#[derive(Clone, Debug)]
pub struct ReportCounter {
    pub name: String,
    pub duration: f64,
    pub counters: Vec<ReportCounter>
}

impl<'a> From<&'a Counter> for ReportCounter {
    fn from(counter: &Counter) -> ReportCounter {
        ReportCounter {
            name: counter.name.to_string(),
            duration: counter.time.unwrap_or(0.0),
            counters : counter.counters.iter().map(From::from).collect()
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameReport {
    pub thread_name: String,
    pub counters: Vec<ReportCounter>,
}

impl FrameReport {
    pub(crate) fn from_thread_data(thread_name: &str, counters: &[Counter]) -> FrameReport {
        FrameReport {
            thread_name : thread_name.to_string(),
            counters : counters.iter().map(From::from).collect()
        }
    }
}

