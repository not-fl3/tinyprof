use time::precise_time_s;

use profiler_frame::*;

pub struct ProfilerState {
    last_frame : usize,
    frames     : Vec<ProfilerFrame>
}

impl ProfilerState {
    pub fn new() -> ProfilerState {
        ProfilerState {
            last_frame : 0,
            frames     : vec![ProfilerFrame::begin_new(precise_time_s())]
        }
    }
    pub fn previous_frame(&mut self) -> &mut ProfilerFrame {
        self.frames.get_mut(self.last_frame - 1).unwrap()
    }

    pub fn current_frame(&mut self) -> &mut ProfilerFrame {
        self.frames.get_mut(self.last_frame).unwrap()
    }
    pub fn next_frame(&mut self) {
        self.frames[self.last_frame].end_time = precise_time_s();
        self.frames[self.last_frame].counters.time = precise_time_s() - self.frames[self.last_frame].begin_time;
        self.last_frame += 1;
        if self.frames.len() <= self.last_frame {
            self.frames.resize(((self.last_frame + 2) as f32 * 1.5) as usize, ProfilerFrame::new());
        }
        self.frames[self.last_frame].begin_time = precise_time_s();
    }
    pub fn begin_track_time(&mut self, name : String) {
        let frame = self.current_frame();
        let counter = frame.counters.active_counter();
        counter.counters.push(ProfilerCounter::new(name));
        counter.active_counter = Some(counter.counters.len() - 1);
    }
    pub fn end_track_time(&mut self, time : f64) {
        let frame = self.current_frame();
        {
            let counter = frame.counters.active_counter();
            counter.time = time;
            frame.covered_time += time;
        }
        {
            let counter = frame.counters.parent_active_counter();
            counter.active_counter = None;
        }
    }

}
