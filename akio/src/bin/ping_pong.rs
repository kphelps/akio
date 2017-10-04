#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;
use std::iter;

struct PongActor {}

#[actor_impl]
impl PongActor {
    #[actor_api]
    pub fn ping(&mut self) {
        self.sender::<PingActor>().pong()
    }

    #[actor_api]
    pub fn ping_n(&mut self, n: u64) {
        println!("{}", n);
    }

    #[on_start]
    fn log_startup(&self) {}

    #[on_stop]
    fn log_shutdown(&self) {}
}

struct PingActor {}

#[actor_impl]
impl PingActor {
    #[actor_api]
    pub fn pong(&mut self) {
        self.sender::<PongActor>().ping()
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
    let pong = spawn(Uuid::new_v4(), PongActor {});
    let ping = spawn(Uuid::new_v4(), PingActor {});
    iter::repeat(())
        .take(20)
        .for_each(|_| pong.ping_with_sender(&ping));
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        iter::repeat(()).take(64).for_each(|_| spawn_ping_loop());
    });
    system.start();
}
