extern crate time;

#[macro_export]
macro_rules! benchmark {
    ($x:expr, $pattern:expr) => {{
        use time::PreciseTime;
        let start = PreciseTime::now();
        let res = $x;
        let end = PreciseTime::now();
        println!($pattern, start.to(end));
        res
    }};
}

#[test]
fn test() {
    let x = benchmark!(2, "2: {}");
    println!("{}", x);
}
