extern crate akio;
extern crate uuid;

use akio::*;
use uuid::Uuid;


struct EchoActor;

impl Actor<String> for EchoActor {
    fn handle_message(&self, message: String) {
        println!("{}", message);
    }
}

impl EchoActor {
    pub fn new() -> Self {
        Self {}
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let mut echo_ref = system.spawn(Uuid::new_v4(), EchoActor::new()).unwrap();
    echo_ref.send("Hello!".to_string());
    system.start();
}
