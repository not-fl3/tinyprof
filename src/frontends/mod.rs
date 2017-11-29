#[cfg(feature = "termion_frontend")]
mod termion_frontend;

#[cfg(feature = "termion_frontend")]
pub use self::termion_frontend::TermionFrontend;

pub trait ProfilerFrontend {
    fn receive_reports(&mut self, report: ::counters::FrameReport);
}
