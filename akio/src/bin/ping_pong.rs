extern crate akio;
extern crate akio_syntax;
extern crate futures;
extern crate uuid;

use akio::*;
use akio_syntax::*;
use futures::Future;
use futures::future::Executor;
use std::iter;
use uuid::Uuid;

actor! {
    PongActor,

    message Ping() {
        PingActor::from_ref(&context.sender).pong(context)
    }
}

actor! {
    PingActor,

    message Pong() {
        PongActor::from_ref(&context.sender).ping(context)
    }
}

fn spawn_ping_loop(system: &mut ActorSystem) {
    let pong_f = PongActor::spawn(system, Uuid::new_v4());
    let ping_f = PingActor::spawn(system, Uuid::new_v4());
    let joint = pong_f.join(ping_f);
    system
        .execute(joint.map(|(pong_ref, ping_ref)| pong_ref.ping_with_sender(&ping_ref)))
        .unwrap();
}

pub fn main() {
    let mut system = ActorSystem::new();
    iter::repeat(())
        .take(1)
        .for_each(|_| { spawn_ping_loop(&mut system); });
    system.start();
}
