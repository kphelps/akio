#![feature(conservative_impl_trait)]
#![feature(fnbox)]
#![feature(proc_macro)]
#![recursion_limit = "1024"]

extern crate akio_syntax;
#[cfg(target_os = "linux")]
extern crate core_affinity;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[cfg(not(target_os = "linux"))]
extern crate num_cpus;
extern crate parking_lot;
extern crate rand;
extern crate tokio_core;
extern crate typemap;
extern crate uuid;

mod actor;
mod actor_cell;
mod actor_container;
mod actor_factory;
mod actor_ref;
mod actor_system;
mod ask_actor;
pub mod context;
mod dispatcher;
pub mod errors;
mod mailbox;
pub mod prelude;

pub use actor::Actor;
pub use actor::MessageHandler;
use actor_cell::ActorCell;
use actor_cell::ActorCellHandle;
use actor_container::ActorContainer;
use actor_factory::create_actor;
pub use actor_ref::ActorRef;
pub use actor_system::ActorSystem;
use dispatcher::Dispatcher;
use mailbox::Mailbox;
use mailbox::MailboxMessage;
use mailbox::SystemMessage;
