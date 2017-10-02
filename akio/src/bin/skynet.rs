#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;

actor! {
    SkynetActor,

    message Poke(n: u64) {
        let sender = self.sender_ref();
        let f: Box<Future<Item = (), Error = ()> + Send> = if n >= 100000 {
            sender.send(n);
            Box::new(future::ok(()))
        } else {
            let fs = (0..10).map(move |sub_n| {
                let next_ref = SkynetActor::spawn(Uuid::new_v4());
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
                          let actor_ref = SkynetActor::spawn(Uuid::new_v4());
                          let f = actor_ref
                              .ask_poke::<u64>(0)
                              .map(|val| println!("Result: {}", val - 1000000));
                          context::execute(f);
                      });
    system.start();
}
