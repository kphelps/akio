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
        self.test_method_calls += 1
    }

    pub fn get_test_method_calls(&mut self) {
        self.reply(self.test_method_calls)
    }
}
