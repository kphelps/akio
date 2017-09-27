extern crate akio;
extern crate uuid;

use akio::*;
use uuid::Uuid;

struct PongActor {
    count: u64,
}

struct PingActor {
    other: ActorRef,
}

impl Actor for PongActor {
    type Message = String;

    fn handle_message(&mut self, context: &ActorContext, message: String) {
        if self.count % 10000 == 0 {
            println!("ping {}", self.count);
        }
        self.count += 1;
        context.sender.send(message, &context.self_ref);
    }
}

impl PongActor {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Actor for PingActor {
    type Message = String;

    fn handle_message(&mut self, context: &ActorContext, message: String) {
        self.other.send(message, &context.self_ref);
    }
}

impl PingActor {
    pub fn new(other: ActorRef) -> Self {
        Self { other: other }
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let pong_ref = system.spawn(Uuid::new_v4(), PongActor::new()).unwrap();
    let ping_ref = system
        .spawn(Uuid::new_v4(), PingActor::new(pong_ref.clone()))
        .unwrap();
    ping_ref.send("ping".to_string(), &pong_ref);
    system.start();
}
