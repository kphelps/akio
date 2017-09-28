extern crate akio;
extern crate uuid;

use akio::*;
use std::iter;
use uuid::Uuid;

struct PongActor {}

struct PingActor {
    other: ActorRef,
}

impl Actor for PongActor {
    type Message = ();

    fn handle_message(&mut self, context: &ActorContext, _message: ()) {
        context.sender.send(());
    }
}

impl PongActor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for PingActor {
    type Message = ();

    fn handle_message(&mut self, context: &ActorContext, _message: ()) {
        self.other.send(());
    }
}

impl PingActor {
    pub fn new(other: ActorRef) -> Self {
        Self { other: other }
    }
}

fn spawn_ping_loop(system: &mut ActorSystem) {
    let pong_ref = system.spawn(Uuid::new_v4(), PongActor::new()).unwrap();
    let ping_ref = system
        .spawn(Uuid::new_v4(), PingActor::new(pong_ref.clone()))
        .unwrap();
    ping_ref.send_from((), &pong_ref);
}

pub fn main() {
    let mut system = ActorSystem::new();
    iter::repeat(1000)
        .take(1)
        .for_each(|_| { spawn_ping_loop(&mut system); });
    system.start();
}
