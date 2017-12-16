#[macro_export]
macro_rules! thread_name {
    ($name:expr) => {
    }
}

#[macro_export]
macro_rules! profile_region {
    ($name:expr) => {
    };
    ($name:expr, $reporter:expr) => {
    };
}

#[macro_export]
macro_rules! print_region_time {
    ($x:expr, $pattern:expr) => {{
    }};
}

#[macro_export]
macro_rules! trace_float {
    ($name:expr, $value:expr) => {{
    }}
}

#[macro_export]
macro_rules! profiler_next_frame {
    () => {{
    }}
}

