extern crate akio;

use akio::prelude::*;
use std::iter;

actor! {
    PongActor,

    state {
        pongs: u64
    }

    message Ping() {
        self.sender::<PingActor>().pong()
    }
}

actor! {
    PingActor,

    state {
        pings: u64 = 0,
    }

    message Pong() {
        self.sender::<PongActor>().ping()
    }
}

fn spawn_ping_loop() {
    let pong = PongActor::spawn(Uuid::new_v4(), 0);
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
    system.on_startup(|| { iter::repeat(()).take(1).for_each(|_| spawn_ping_loop()); });
    system.start();
}
