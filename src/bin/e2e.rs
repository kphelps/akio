extern crate akio;
extern crate futures;
extern crate uuid;

use akio::*;
use futures::Future;
use futures::future::Executor;
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
    let pong_f = system.spawn(Uuid::new_v4(), PongActor::new());
    let ping_f = system.spawn(Uuid::new_v4(), PingActor::new());
    let joint = pong_f.join(ping_f);
    system
        .execute(joint.map(|(pong_ref, ping_ref)| ping_ref.send((), &pong_ref)))
        .unwrap();
}

pub fn main() {
    let mut system = ActorSystem::new();
    iter::repeat(())
        .take(1000)
        .for_each(|_| { spawn_ping_loop(&mut system); });
    system.start();
}
