extern crate akio;
extern crate uuid;

use akio::*;
use std::iter;
use uuid::Uuid;

struct PongActor {}

struct PingActor {}

impl Actor for PongActor {
    type Message = ();

    fn handle_message(&mut self, context: &ActorContext, _message: ()) {
        context.reply(())
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
        context.reply(())
    }
}

impl PingActor {
    pub fn new() -> Self {
        Self {}
    }
}

fn spawn_ping_loop(system: &mut ActorSystem) {
    let pong_ref = system.spawn(Uuid::new_v4(), PongActor::new()).unwrap();
    let ping_ref = system.spawn(Uuid::new_v4(), PingActor::new()).unwrap();
    ping_ref.send((), &pong_ref);
}

pub fn main() {
    let mut system = ActorSystem::new();
    iter::repeat(1000)
        .take(1)
        .for_each(|_| { spawn_ping_loop(&mut system); });
    system.start();
}
