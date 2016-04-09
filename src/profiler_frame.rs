
#[derive(Clone)]
pub struct ProfilerCounter {
    pub name           : String,
    pub time           : f64,
    pub counters       : Vec<ProfilerCounter>,
    pub active_counter : Option<usize>
}

impl ProfilerCounter {
    pub fn new(name : String) -> ProfilerCounter {
        ProfilerCounter {
            name           : name.to_string(),
            time           : 0.0,
            counters       : Vec::new(),
            active_counter : None
        }
    }
    pub fn active_counter(&mut self) -> &mut ProfilerCounter {
        match self.active_counter {
            None => self,
            Some(i) => self.counters[i].active_counter()
        }
    }
    pub fn parent_active_counter(&mut self) -> &mut ProfilerCounter {
        match self.active_counter {
            None => self,
            Some(i) if self.counters[i].active_counter.is_none() => { self }
            Some(i) => { self.counters[i].parent_active_counter() }
        }
    }

}

#[derive(Clone)]
pub struct ProfilerFrame {
    pub begin_time     : f64,
    pub end_time       : f64,
    pub covered_time   : f64,
    pub counters       : ProfilerCounter
}
impl ProfilerFrame {
    pub fn begin_new(time : f64) -> ProfilerFrame {
        ProfilerFrame {
            begin_time     : time,
            end_time       : 0.0,
            covered_time   : 0.0,
            counters       : ProfilerCounter::new("root".to_string())
        }
    }
    pub fn new() -> ProfilerFrame {
        ProfilerFrame::begin_new(0.0)
    }
}
