#![cfg_attr(not(feature="prof"),  allow(dead_code))]

use std::sync::mpsc;
use failure::Error;
use counters::FrameReport;

#[derive(Debug, Fail)]
enum ProfilerError {
    #[fail(display = "Profiler not yet initialised!")] NotInitialised,
}

const PROFILER_CHANNEL_BOUND: usize = 10000;

struct ProfilerStateSender(*const mpsc::SyncSender<FrameReport>);
unsafe impl Sync for ProfilerStateSender {}

static mut PROFILER_STATE_SENDER: ProfilerStateSender = ProfilerStateSender(0 as *const _);

pub(crate) fn send_profiler_state(state: FrameReport) -> Result<(), Error> {
    let sender = unsafe { PROFILER_STATE_SENDER.0.as_ref() };

    sender
        .ok_or(ProfilerError::NotInitialised)?
        .try_send(state)?;
    Ok(())
}

pub struct Profiler {
    receiver: mpsc::Receiver<FrameReport>,
}

impl Profiler {
    pub fn new() -> Profiler {
        let (sender, receiver) = mpsc::sync_channel(PROFILER_CHANNEL_BOUND);

        unsafe {
            let r = &mut PROFILER_STATE_SENDER.0;
            *r = Box::into_raw(Box::new(sender.clone()));
        }

        Profiler { receiver }
    }

    /// Non blocking read receiving channel while its not empty.
    pub fn receive_reports(&self) -> Vec<FrameReport> {
        let mut reports = vec![];

        while let Ok(report) = self.receiver.try_recv() {
            reports.push(report);
        }
        reports
    }
}
