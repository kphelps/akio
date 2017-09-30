#[cfg(target_os = "linux")]
extern crate core_affinity;
extern crate futures;
#[cfg(not(target_os = "linux"))]
extern crate num_cpus;
extern crate tokio_core;
extern crate uuid;

mod actor;
mod actor_cell;
mod actor_factory;
mod actor_ref;
mod actor_system;
mod actor_supervisor;
mod context;
mod dispatcher;
mod mailbox;

pub use actor::Actor;
pub use actor::BaseActor;
use actor_cell::ActorCell;
use actor_cell::ActorCellHandle;
pub use actor_factory::ActorFactory;
pub use actor_factory::ActorChildren;
pub use actor_ref::ActorRef;
pub use actor_system::ActorEvent;
pub use actor_system::ActorSystem;
pub use actor_supervisor::ActorSupervisor;
pub use context::ActorContext;
use dispatcher::Dispatcher;
use mailbox::Mailbox;
use mailbox::MailboxMessage;
