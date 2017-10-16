use akio::prelude::*;

pub struct TestActor {
    test_method_calls: u64,
}

impl TestActor {
    pub fn new() -> Self {
        Self {
            test_method_calls: 0,
        }
    }
}

#[actor_impl]
impl TestActor {
    #[actor_api]
    pub fn test_method(&mut self) {
        self.test_method_calls += 1;
        self.done()
    }

    #[actor_api]
    pub fn get_test_method_calls(&mut self) -> u64 {
        self.respond(self.test_method_calls)
    }
}
