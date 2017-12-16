use time::precise_time_s;

/// Time measurement trait
pub trait CustomReporter {
    /// Called on region begin
    /// Using to start some internal time counter
    fn start(&mut self);

    /// Called on region end
    fn end(&mut self);

    /// Called on region end, should return passed time
    /// This will be called only once, at the end of the frame.
    /// None means that reporter is not ready yet.
    fn duration(&mut self) -> Option<f64>;
}

pub(crate) mod reporter_id {
    pub const SYSTEM_CLOCK: usize = 0;
    pub const CUSTOM: usize = 1;
}

/// Source of time measure data
/// Time - measure start time and end time from system clock
/// Custom - provide object responsible on time measurement. Good place to put your glQueries
pub enum Reporter {
    SystemClock {
        start_time: f64,
        end_time: f64
    },
    Custom(Box<CustomReporter>)
}

impl Reporter {
    pub fn start(&mut self) {
        match self {
            &mut Reporter::SystemClock{ref mut start_time, ..} => {
                *start_time = precise_time_s();
            },
            &mut Reporter::Custom(ref mut reporter) => {
                reporter.start();
            }
        }
    }
    pub fn end(&mut self) {
        match self {
            &mut Reporter::SystemClock{ref mut end_time, ..} => {
                *end_time = precise_time_s();
            },
            &mut Reporter::Custom(ref mut reporter) => {
                reporter.end()
            }
        }
    }
    pub fn duration(&mut self) -> Option<f64> {
        match self {
            &mut Reporter::SystemClock{end_time, start_time} => {
                Some(end_time - start_time)
            },
            &mut Reporter::Custom(ref mut reporter) => {
                reporter.duration()
            }
        }
    }
}
