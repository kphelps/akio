extern crate akio;
extern crate akio_syntax;
extern crate futures;
extern crate uuid;

use akio::*;
use akio_syntax::*;
use futures::Future;
use futures::future;
use std::iter;
use uuid::Uuid;

actor! {
    PongActor,

    message Ping() {
        Box::new(future::ok(self.sender::<PingActor>().pong()))
    }
}

actor! {
    PingActor,

    message Pong() {
        Box::new(future::ok(self.sender::<PongActor>().ping()))
    }
}

fn spawn_ping_loop() -> Box<Future<Item = (), Error = ()>> {
    let pong_f = PongActor::spawn(Uuid::new_v4());
    let ping_f = PingActor::spawn(Uuid::new_v4());
    let joint = pong_f.join(ping_f);
    Box::new(joint.map(|(pong_ref, ping_ref)| pong_ref.ping_with_sender(&ping_ref)))
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| -> Box<Future<Item = (), Error = ()>> {
                          let fs = iter::repeat(()).take(1).map(|_| spawn_ping_loop());
                          Box::new(future::join_all(fs).map(|_| ()))
                      });
    system.start();
}
