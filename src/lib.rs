extern crate time;
extern crate failure;
extern crate vec_map;
#[macro_use]
extern crate failure_derive;
#[cfg(feature = "termion_frontend")]
extern crate termion;
#[cfg(feature = "termion_frontend")]
extern crate gag;

mod frontends;
mod thread_storage;
mod profiler;
mod region;
mod reporter;
mod counters;

#[cfg(feature="prof")]
#[macro_use] mod macros;
#[cfg(not(feature="prof"))]
#[macro_use] mod macros_dummy;

pub use counters::FrameReport;
pub use profiler::Profiler;
pub use thread_storage::{next_frame, set_thread_name, set_variable_value};
pub use region::Region;
pub use reporter::{Reporter, CustomReporter};
pub use frontends::ProfilerFrontend;
#[cfg(feature = "termion_frontend")]
pub use frontends::TermionFrontend;
