use thread_storage;
use reporter::Reporter;

pub struct Region {
    pub reporter_id : usize,
    pub id : &'static str
}

impl Region {
    pub fn new(name: &str, id : &'static str, reporter: Reporter, reporter_id : usize) -> Region {
        thread_storage::start_region(name, id, reporter, reporter_id);

        Region {
            reporter_id,
            id
        }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        thread_storage::end_region(self.reporter_id);
    }
}
