extern crate akio;
extern crate uuid;

use akio::*;
use uuid::Uuid;


struct EchoActor {
    count: u64,
}

struct PingActor {
    other: ActorRef<String>,
}

impl Actor<String> for EchoActor {
    fn handle_message(&mut self, context: &ActorContext<String>, message: String) {
        if self.count % 10000 == 0 {
            println!("{}", self.count);
        }
        self.count += 1;
        context.self_ref.send("".to_string());
    }
}

impl EchoActor {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Actor<String> for PingActor {
    fn handle_message(&mut self, _context: &ActorContext<String>, message: String) {
        println!("proxying");
        self.other.send(message);
    }
}

impl PingActor {
    pub fn new(other: ActorRef<String>) -> Self {
        Self { other: other }
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    let echo_ref = system.spawn(Uuid::new_v4(), EchoActor::new()).unwrap();
    let ping_ref = system
        .spawn(Uuid::new_v4(), PingActor::new(echo_ref))
        .unwrap();
    ping_ref.send("Hello!".to_string());
    system.start();
}
