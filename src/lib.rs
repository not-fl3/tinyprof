extern crate time;
#[macro_use] extern crate lazy_static;

mod profiler_frame;
mod profiler_state;

use std::sync::Mutex;
use time::precise_time_s;
use std::ops::Drop;

pub use profiler_frame::*;
pub use profiler_state::*;

lazy_static! {
    pub static ref PROFILER_STATE: Mutex<ProfilerState> = Mutex::new(ProfilerState::new());
}

#[derive(Debug)]
pub struct ProfilerReport {
    pub name        : String,
    pub time        : f64,
    pub percent     : f64,
    pub sub_reports : Vec<ProfilerReport>
}

pub fn profiler_report() -> ProfilerReport {
    state_report(&mut *PROFILER_STATE.lock().unwrap())
}
pub fn profiler_next_frame() {
    PROFILER_STATE.lock().unwrap().next_frame();
}

pub struct ProfilerRegion {
    pub enter_time : f64,
    pub name       : String
}
impl ProfilerRegion {
    pub fn new(name : &str) -> ProfilerRegion {
        PROFILER_STATE.lock().unwrap().begin_track_time(name.to_string());
        ProfilerRegion {
            enter_time : precise_time_s(),
            name       : name.to_string()
        }
    }
}
impl Drop for ProfilerRegion {
    fn drop(&mut self) {
        PROFILER_STATE.lock().unwrap().end_track_time(precise_time_s() - self.enter_time);
    }
}


#[macro_export]
macro_rules! profile_region {
    ($name:expr) => {
        let _profile = $crate::ProfilerRegion::new($name);
    }
}

#[macro_export]
macro_rules! print_region_time {
    ($x:expr, $pattern:expr) => {{
        use time::precise_time_s;

        let start = precise_time_s();
        let res = $x;
        let end = precise_time_s();
        println!($pattern, end - start);
        res
    }};
}

pub fn state_report(state : &mut ProfilerState) -> ProfilerReport {
    let frame = state.previous_frame();
    report(&frame.counters, frame.end_time - frame.begin_time)
}

fn report(counter : &ProfilerCounter, time : f64) -> ProfilerReport {
    ProfilerReport {
        name        : counter.name.clone(),
        time        : counter.time,
        percent     : counter.time / time * 100.0,
        sub_reports : counter.counters.iter().map(|x| report(x, counter.time)).collect()
    }
}

#[test]
fn test_print_region_time() {
    let _ : i32 = print_region_time!({
        (1 ..).take(1000).fold(0, |x, y| x + y)
    }, "Something time tooked: {}");
}

#[test]
fn test_track_regions() {
    {
        profile_region!("Main region");

        (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);

        {
            profile_region!("subregion 1");
            (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);
        }

        {
            profile_region!("subregion 2");
            (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);

            {
                profile_region!("subsubregion 1");
                (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);
            }

            {
                profile_region!("subsubregion 2");
                (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);
            }

        }

        {
            profile_region!("subregion 3");
            (1 ..).take(500000).fold(0, |x, y| ((x * y) as f32).sqrt().sin() as i32 + 1);
        }
    }

    profiler_next_frame();

    println!("{:?}", profiler_report());
}
