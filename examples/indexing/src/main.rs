#![feature(conservative_impl_trait)]
#![feature(proc_macro)]

extern crate akio;

use akio::prelude::*;

struct OrganizationActor {
    name: String,
    owner: String,
}

#[actor_impl]
impl OrganizationActor {
    pub fn new(name: String, owner: String) -> Self {
        Self {
            name: name,
            owner: owner,
        }
    }

    #[actor_api]
    pub fn debug(&mut self) {
        println!(
            "id: {}, name: {}, owner: {}",
            self.id(),
            self.name,
            self.owner
        )
    }
}

struct OrganizationsActor {}

#[actor_impl]
impl OrganizationsActor {
    pub fn new() -> Self {
        Self {}
    }

    #[actor_api]
    pub fn add(&mut self, org_id: Uuid, name: String, owner: String) {
        println!("Add {}", org_id);
        let org_actor = OrganizationActor::new(name, owner);
        let org_ref = spawn(org_id, org_actor);
        org_ref.debug()
    }

    #[actor_api]
    pub fn count(&mut self) {
        let org_count = self.with_children(|children| children.iter().len());
        println!("org count: {}", org_count)
    }
}

pub fn main() {
    let mut system = ActorSystem::new();
    system.on_startup(|| {
        let orgs = spawn(Uuid::new_v4(), OrganizationsActor::new());
        orgs.add(
            Uuid::new_v4(),
            "org 1".to_string(),
            "kphelps@salsify.com".to_string(),
        );
        orgs.add(
            Uuid::new_v4(),
            "org 2".to_string(),
            "kylep91@gmail.com".to_string(),
        );
        orgs.add(
            Uuid::new_v4(),
            "org 3".to_string(),
            "meh@meh.meh".to_string(),
        );
        orgs.count();
    });
    system.start();
}
