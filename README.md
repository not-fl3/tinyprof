# tinyprof

Simple region based profiler. Running with application and displaying profiling information to embedded frontend.

![imgui!](https://i.imgur.com/rqZB5pN.gif)
![imgui!](https://i.imgur.com/zKyTvBL.gif)

# Goals

- No overhead or any additional code when compiled-out.
- Miminimum impact on profiled code performance.
- Simple api for regions.
- Multithreading.

# Non-goals

- Profiler visualisation perfomance is not a goal. If it not impact relative profiling data - it's okay if visualizer takes some FPS on main thread.

# How to use


Without "prof" feature tinyprof will give your application zero overhead - all macros will be exposed to nothing.

## For libraries

Add one crate to dependencies - ```tinyprof```
Add feature named "prof" to your Cargo.toml, like this:
```
[features]
prof = []
```
Now you could just use all tinyprof's macros anywhere.

## For binaries

Add two crates - ```tinyprof``` and ```tinyprof_frontends```
Turn on "prof" feature, something like this:
```
[features]
prof = [
     "lib1/prof",
     "lib2/prof",
     "tinyprof/prof",
     "tinyprof_frontends/termion_frontend" # if you need frontend
]
```

Initialise profiler and frontend.
```rust
#[cfg(feature = "prof")]
let (profiler, mut frontend) = (
    prof::Profiler::new(),
    frontends::TermionFrontend::new(Default::default()),
);
```

And put ```profile_region!``` and other tinyprof's macros to suspisiously parts of your code.


```rust
{

    profile_region!("Root region");

    // something heavy

    {
        profiler_region!("sub region 1");
        // something heavy
    }

    {
        profiler_region!("sub region 2");
        // something heavy
    }
}
profiler_next_frame!();
```

Grab current frame information and feed it to frontend. Termion frontend have ```draw``` method to display profiling information to console.

```rust
...
#[cfg(feature="prof")]
{
    use tinyprof::ProfilerFrontend;

    for report in profiler.receive_reports().into_iter() {
        frontend.receive_reports(report);
    }
    if frontend.draw() == false {
        return;
    }
}
...

```

Check "examples/profiling.rs" for more details.

