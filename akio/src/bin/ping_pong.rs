#![feature(conservative_impl_trait)]
#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;
use std::iter;

struct PongActor {
    ping: ActorRef<PingActor>,
}

#[actor_impl]
impl PongActor {
    pub fn new(ping: ActorRef<PingActor>) -> Self {
        Self {
            ping: ping
        }
    }

    #[actor_api]
    pub fn ping(&mut self) {
        self.ping.pong();
    }

    #[on_start]
    fn log_startup(&self) {}

    #[on_stop]
    fn log_shutdown(&self) {}
}

struct PingActor {
    pong: Option<ActorRef<PongActor>>
}

#[actor_impl]
impl PingActor {
    pub fn new() -> Self {
        Self {
            pong: None,
        }
    }

    #[actor_api]
    pub fn initialize(&mut self, pong: ActorRef<PongActor>) {
        self.pong = Some(pong);
    }

    #[actor_api]
    pub fn pong(&mut self) {
        self.pong.as_ref().map(|pong| pong.ping());
    }

    #[on_start]
    fn on_start(&self) {
        println!("Starting ping!");
    }

    #[on_stop]
    fn on_stop(&self) {
        println!("Stopping ping!");
    }
}

fn spawn_ping_loop() {
    let ping = PingActor::new().start();
    let pong = PongActor::new(ping.clone()).start();
    ping.initialize(pong.clone());
    iter::repeat(())
        .take(20)
        .for_each(|_| pong.ping());
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        iter::repeat(()).take(64).for_each(|_| spawn_ping_loop());
    });
    system.start();
}
