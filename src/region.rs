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
