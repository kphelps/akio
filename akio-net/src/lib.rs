#![feature(conservative_impl_trait)]

extern crate bytes;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate protobuf;
extern crate parking_lot;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;

mod client;
mod client_state;
mod errors;
mod ipc;
mod node;
mod protocol;
mod server;

pub use node::RemoteNode;
