use time::precise_time_s;

/// Time measurement trait
pub trait CustomReporter {
    /// Called on region begin
    /// Using for start some internal time counter
    fn start(&mut self);

    /// Called on region end, should return passed time
    /// This will be called only once, on region end
    fn duration(&mut self) -> f64;
}

pub(crate) mod reporter_id {
    pub const SYSTEM_CLOCK: usize = 0;
    pub const CUSTOM: usize = 1;
}

/// Source of time measure data
/// Time - measure start time and end time from system clock
/// Custom - provide object responsible on time measurement. Good place to put your glQueries
pub enum Reporter {
    SystemClock(f64),
    Custom(Box<CustomReporter>)
}

impl Reporter {
    pub fn start(&mut self) {
        match self {
            &mut Reporter::SystemClock(ref mut time) => {
                *time = precise_time_s();
            },
            &mut Reporter::Custom(ref mut reporter) => {
                reporter.start();
            }
        }
    }
    pub fn end(&mut self) -> f64 {
        match self {
            &mut Reporter::SystemClock(ref mut time) => {
                precise_time_s() - *time
            },
            &mut Reporter::Custom(ref mut reporter) => {
                reporter.duration()
            }
        }
    }
}
