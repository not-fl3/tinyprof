#[cfg(feature = "termion_frontend")]
extern crate termion;
#[cfg(feature = "termion_frontend")]
extern crate gag;
#[cfg(feature = "imgui_frontend")]
extern crate imgui;
extern crate lazysort;

extern crate tinyprof;

#[cfg(feature = "termion_frontend")]
mod termion_frontend;
#[cfg(feature = "imgui_frontend")]
mod imgui_frontend;

#[cfg(feature = "termion_frontend")]
pub use self::termion_frontend::TermionFrontend;
#[cfg(not(feature="termion_frontend"))]
pub type TermionFrontend = ();

#[cfg(feature = "imgui_frontend")]
pub use self::imgui_frontend::ImguiFrontend;
#[cfg(not(feature="imgui_frontend"))]
pub type ImguiFrontend = ();

pub trait ProfilerFrontend {
    fn receive_reports(&mut self, report: tinyprof::FrameReport);
}

