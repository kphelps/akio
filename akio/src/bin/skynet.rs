#![feature(conservative_impl_trait)]
#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;

struct SkynetActor;

#[actor_impl]
impl SkynetActor {
    pub fn new() -> Self {
        Self {
        }
    }

    #[actor_api]
    pub fn poke(&mut self, n: u64) -> u64
    {
        if n >= 100000 {
            future::ok(n)
        } else {
            let fs = (0..10).map(move |sub_n| {
                let next_ref = SkynetActor::new().start();
                let id = n * 10 + sub_n + 1;
                next_ref.poke(id)
            });
            future::join_all(fs).map(move |vals| {
                let pre_sum: u64 = vals.iter().sum();
                let sum: u64 = pre_sum + n;
                sum
            })
        }
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        let actor_ref = SkynetActor::new().start();
        let f = actor_ref
            .poke(0)
            .map(|val| println!("Result: {}", val - 1000000));
        context::execute(f);
    });
    system.start();
}
