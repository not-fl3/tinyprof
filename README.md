# tinyprof

Simple region based profiler. Running with application and displaying profiling information to embedded frontend.

# How to use


Without "prof" feature tinyprof will give your application zero overhead - all macros will be exposed to nothing.

```
[features]
prof = ["tinyprof/prof", "tinyprof/termion_frontend"]
```

Initialise profiler and frontend. It will be just (None, None) without "prof" feature.
```rust
let (mut profiler, mut frontend) = init_profiler!(TermionFrontend::new(Default::default()));

```

Put ```profile_region!``` to suspisiously parts of your code.


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
if cfg!(feature="prof") {
   let profiler : &mut Profiler = profiler.as_mut().unwrap();
   let frontend : &mut TermionFrontend = frontend.as_mut().unwrap();

   for report in profiler.receive_reports().into_iter() {
       frontend.receive_reports(report);
   }
   if frontend.draw() == false {
       return;
   }
}

```

