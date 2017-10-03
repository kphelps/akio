#![feature(proc_macro)]
extern crate akio;

mod common;

use akio::prelude::*;
use common::*;

#[test]
fn test_create_actor_id() {
    with_actor_system(|_| {
                          let id = Uuid::new_v4();
                          let actor_ref = spawn(id.clone(), TestActor::new());
                          assert_eq!(id, actor_ref.id());
                      })
}

#[test]
fn test_create_get_actor() {
    with_actor_system(|system| {
                          let id = Uuid::new_v4();
                          let actor_ref = spawn(id.clone(), TestActor::new());
                          let system_actor_ref = system.get_actor(&id);
                          assert!(system_actor_ref.is_some());
                          assert_eq!(system_actor_ref.unwrap().id(), actor_ref.id());
                      })
}

#[test]
fn test_create_stop_get_actor() {
    with_actor_system_async(|system| {
        let id = Uuid::new_v4();
        let actor_ref = spawn(id.clone(), TestActor::new());
        let f = actor_ref
            .clone()
            .stop()
            .map_err(|_| ())
            .map(move |_| {
                     assert!(system.get_actor(&id).is_none());
                     assert!(!actor_ref.exists());
                 });
        Box::new(f)
    })
}
