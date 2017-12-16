#[macro_export]
macro_rules! thread_name {
    ($name:expr) => {
        $crate::set_thread_name($name);
    }
}

#[macro_export]
macro_rules! profile_region {
    ($name:expr) => {
        let _profile = if cfg!(feature = "prof") {
            Some($crate::Region::new($name,
                                     concat!(file!(), " ", column!(), " ", line!()),
                                     $crate::Reporter::SystemClock {
                                         start_time: 0.0,
                                         end_time: 0.0
                                     },
                                     0))
        } else {
            None
        };
    };
    ($name:expr, $reporter:expr) => {
        let _profile = if cfg!(feature = "prof") {
            Some($crate::Region::new($name,
                                     concat!(file!(), " ", column!(), " ", line!()),
                                     $crate::Reporter::Custom($reporter),
                                     1))
        } else {
            None
        };
    };
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

#[macro_export]
macro_rules! trace_float {
    ($name:expr, $value:expr) => {{
        $crate::set_variable_value($name.to_string(), $value)
    }}
}

#[macro_export]
macro_rules! profiler_next_frame {
    () => {{
        $crate::next_frame().unwrap();
    }}
}
