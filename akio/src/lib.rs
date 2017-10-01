#![feature(fnbox)]

#[cfg(target_os = "linux")]
extern crate core_affinity;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[cfg(not(target_os = "linux"))]
extern crate num_cpus;
extern crate parking_lot;
extern crate rand;
extern crate tokio_core;
extern crate uuid;

mod actor;
mod actor_cell;
mod actor_factory;
mod actor_ref;
mod actor_system;
mod actor_supervisor;
pub mod context;
mod dispatcher;
mod mailbox;

pub use actor::Actor;
pub use actor::BaseActor;
pub use actor::TypedActor;
use actor_cell::ActorCell;
pub use actor_factory::ActorFactory;
pub use actor_factory::ActorChildren;
pub use actor_ref::ActorRef;
pub use actor_system::ActorSystem;
pub use actor_supervisor::ActorSupervisor;
pub use context::ActorContext;
use dispatcher::Dispatcher;
use mailbox::Mailbox;
use mailbox::MailboxMessage;
