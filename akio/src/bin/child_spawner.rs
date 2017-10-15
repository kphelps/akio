#![feature(conservative_impl_trait)]
#![feature(proc_macro)]
extern crate akio;

use akio::prelude::*;

struct TelephoneActor {
    children: Vec<ActorRef<TelephoneActor>>,
}

#[actor_impl]
impl TelephoneActor {
    pub fn new() -> Self {
        Self {
            children: Vec::new()
        }
    }

    #[actor_api]
    pub fn spawn_next(&mut self, n: u64) {
        if n % 10000 == 0 {
            println!("Spawning {}", n);
        }
        let next_ref = TelephoneActor::new().start();
        next_ref.spawn_next(n + 1);
        self.children.push(next_ref);
    }

    #[actor_api]
    pub fn message(&mut self, msg: String) {
        self.children.iter().for_each(|child| {
            child.message(msg.clone())
        })
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        let actor_ref = TelephoneActor::new().start();
        actor_ref.spawn_next(0);
        actor_ref.message("Yo".to_string());
    });
    system.start();
}
