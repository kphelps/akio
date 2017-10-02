extern crate akio;

use akio::prelude::*;
use std::iter;

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
    let pong = PongActor::spawn(Uuid::new_v4());
    let ping = PingActor::spawn(Uuid::new_v4());
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
    pong.ping_with_sender(&ping);
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| { iter::repeat(()).take(64).for_each(|_| spawn_ping_loop()); });
    system.start();
}
