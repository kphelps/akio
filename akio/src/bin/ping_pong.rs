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

fn spawn_ping_loop() {
    println!("Exec?");
    let pong = PongActor::spawn(Uuid::new_v4());
    let ping = PingActor::spawn(Uuid::new_v4());
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    println!("Exec?");
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| -> Box<Future<Item = (), Error = ()>> {
                          iter::repeat(()).take(1).for_each(|_| spawn_ping_loop());
                          Box::new(future::ok(()))
                      });
    system.start();
}
