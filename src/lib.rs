extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate time;
extern crate vec_map;

mod thread_storage;
mod profiler;
mod region;
mod reporter;
mod counters;

#[cfg(feature = "prof")]
#[macro_use]
mod macros;
#[cfg(not(feature = "prof"))]
#[macro_use]
mod macros_dummy;

pub use counters::{FrameReport, ReportCounter};
#[cfg(feature = "prof")]
pub use profiler::Profiler;
#[cfg(not(feature = "prof"))]
pub type Profiler = ();
pub use thread_storage::{next_frame, set_thread_name, set_variable_value};
pub use region::Region;
pub use reporter::{CustomReporter, Reporter};
