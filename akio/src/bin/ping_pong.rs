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
        self.sender::<PingActor>().pong()
    }
}

actor! {
    PingActor,

    message Pong() {
        self.sender::<PongActor>().ping()
    }
}

fn spawn_ping_loop() {
    let pong_f = PongActor::spawn(Uuid::new_v4());
    let ping_f = PingActor::spawn(Uuid::new_v4());
    let joint = pong_f.join(ping_f);
    context::execute(joint.map(|(pong_ref, ping_ref)| pong_ref.ping_with_sender(&ping_ref)))
        .unwrap();
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
                          iter::repeat(())
                              .take(1)
                              .for_each(|_| { spawn_ping_loop(); });
                      });
    system.start();
}
