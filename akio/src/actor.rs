use super::ActorRef;
use futures::future;
use futures::prelude::*;
use std::any::Any;

pub trait BaseActor: Send {
    fn handle_any(&mut self, message: Box<Any + Send>) -> Box<Future<Item = (), Error = ()>>;
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, message: Self::Message) -> Box<Future<Item = (), Error = ()>>;
}

impl<T> BaseActor for T
    where T: Actor + Send
{
    fn handle_any(&mut self, any_message: Box<Any + Send>) -> Box<Future<Item = (), Error = ()>> {
        match any_message.downcast() {
            Ok(message) => self.handle_message(*message),
            _ => {
                println!("Unhandled message");
                Box::new(future::ok(()))
            }
        }
    }
}

pub trait TypedActor {
    type RefType;

    fn from_ref(actor_ref: &ActorRef) -> Self::RefType;
}
