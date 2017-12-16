use reporter::Reporter;

use std::collections::HashMap;

pub(crate) struct Counter {
    pub(crate) name: String,
    pub(crate) id: &'static str,
    pub(crate) reporter: Reporter,
    pub(crate) counters: Vec<Counter>,
    pub(crate) parent: *mut Counter,
}

#[derive(Clone, Debug)]
pub struct ReportCounter {
    /// Name of profiling region, provided by user in profile_region!().
    pub name: String,
    /// Unique region id. Actually its filename + line of profile_region!() invocation.
    pub id: &'static str,
    /// Seconds spent on that region
    /// None means that region is not finished yet.
    pub duration: Option<f64>,
    /// Counters occured inside that region.
    pub counters: Vec<ReportCounter>,
}

impl<'a> From<&'a mut Counter> for ReportCounter {
    fn from(counter: &mut Counter) -> ReportCounter {
        ReportCounter {
            name: counter.name.to_string(),
            id: counter.id,
            duration: counter.reporter.duration(),
            counters: counter.counters.iter_mut().map(From::from).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameReport {
    pub thread_name: String,
    pub frame: i32,
    pub counters: Vec<ReportCounter>,
    pub variables: HashMap<String, f32>,
}

impl FrameReport {
    pub(crate) fn from_thread_data(
        thread_name: &str,
        frame: i32,
        counters: &mut [Counter],
        variables: &HashMap<String, f32>,
    ) -> FrameReport {
        FrameReport {
            frame: frame,
            variables: variables.clone(),
            thread_name: thread_name.to_string(),
            counters: counters.iter_mut().map(From::from).collect(),
        }
    }
}
