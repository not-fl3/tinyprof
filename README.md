# tinyprof

Simple region based profiler.

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

    profiler_next_frame();
}
```
