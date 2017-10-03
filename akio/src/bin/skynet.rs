#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;

struct SkynetActor;

#[actor_impl]
impl SkynetActor {
    #[actor_api]
    pub fn poke(&mut self, n: u64) {
        let sender = self.sender_ref();
        let f: Box<Future<Item = (), Error = ()> + Send> = if n >= 100000 {
            sender.send(n);
            Box::new(future::ok(()))
        } else {
            let fs = (0..10).map(move |sub_n| {
                                     let next_ref = spawn(Uuid::new_v4(), SkynetActor {});
                                     let id = n * 10 + sub_n + 1;
                                     next_ref.ask_poke::<u64>(id)
                                 });
            let f = future::join_all(fs).map(move |vals| {
                let pre_sum: u64 = vals.iter().sum();
                let sum: u64 = pre_sum + n;
                sender.send(sum);
            });
            Box::new(f)
        };
        context::execute(f);
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
                          let actor_ref = spawn(Uuid::new_v4(), SkynetActor {});
                          let f = actor_ref
                              .ask_poke::<u64>(0)
                              .map(|val| println!("Result: {}", val - 1000000));
                          context::execute(f);
                      });
    system.start();
}
