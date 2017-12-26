extern crate rand;
#[macro_use]
extern crate tinyprof;
extern crate tinyprof_termion as frontend;

use std::thread;

use tinyprof as prof;

fn do_work() {
    let time = (rand::random::<f64>() * 50.0) as u64;
    ::std::thread::sleep(::std::time::Duration::from_millis(time));
}

fn main() {
    #[cfg(feature = "prof")]
    let (profiler, mut frontend) = (
        prof::Profiler::new(),
        frontend::TermionFrontend::new(Default::default()),
    );

    thread::spawn(|| {
        loop {
            thread_name!("Thread 1");

            {
                profile_region!("sub region 1");

                {
                    profile_region!("sub region 1");
                    do_work();
                }

                {
                    profile_region!("sub region 2");
                    do_work();

                    {
                        profile_region!("sub sub region 1");
                        do_work();
                    }

                    trace_float!("random number", rand::random::<i8>() as f32);
                    {
                        profile_region!("sub sub region 2");
                        do_work();
                    }
                }
            }

            {
                profile_region!("sub region 2");

                {
                    profile_region!("sub sub region 1");
                    do_work();
                }

                {
                    profile_region!("sub sub region 2");
                    do_work();
                }

                for _ in 0..10 {
                    profile_region!("repeating subregion");
                    do_work();
                }
            }

            profiler_next_frame!();
        }
    });

    thread::spawn(|| {
        loop {
            println!("Just random println {}", rand::random::<u8>());
            prof::set_thread_name("Thread 2");
            {
                profile_region!("long computation");
                for _ in 0..4 {
                    do_work();
                }
            }

            {
                profile_region!("another long  computations");
                for _ in 0..8 {
                    do_work();
                }
            }
            do_work();

            profiler_next_frame!();
        }
    });

    loop {
        #[cfg(feature = "prof")]
        {
            for report in profiler.receive_reports().into_iter() {
                frontend.receive_reports(report);
            }
            if frontend.draw() == false {
                return;
            }
        }
    }
}
