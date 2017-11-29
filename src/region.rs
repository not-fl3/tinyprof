use thread_storage;
use reporter::Reporter;

pub struct Region {
    pub reporter_id : usize
}

impl Region {
    pub fn new(name : &str, reporter: Reporter, reporter_id : usize) -> Region {
        thread_storage::start_region(name, reporter, reporter_id);

        Region {
            reporter_id
        }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        thread_storage::end_region(self.reporter_id);
    }
}


#[macro_export]
macro_rules! profile_region {
    ($name:expr) => {
        let _profile = if cfg!(feature = "prof") {
            Some($crate::Region::new($name, $crate::Reporter::SystemClock(0.0), 0))
        } else {
            None
        };
    };
    ($name:expr, $reporter:expr) => {
        let _profile = if cfg!(feature = "prof") {
           Some($crate::Region::new($name, $crate::Reporter::Custom($reporter), 1))
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
