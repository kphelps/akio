#![feature(conservative_impl_trait)]

extern crate bytes;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate futures;
extern crate futures_cpupool;
extern crate protobuf;
extern crate parking_lot;
extern crate rand;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_timer;
extern crate uuid;

mod client;
mod client_state;
mod errors;
mod ipc;
mod node;
mod protocol;
mod rpc;
mod server;

pub use node::RemoteNode;
