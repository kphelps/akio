use std::any::Any;
use super::ActorContext;

pub trait BaseActor {
    fn handle_any(&mut self, context: &ActorContext, message: Box<Any>);
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, context: &ActorContext, message: Self::Message);
}

impl<T> BaseActor for T
    where T: Actor
{
    fn handle_any(&mut self, context: &ActorContext, any_message: Box<Any>) {
        match any_message.downcast() {
            Ok(message) => self.handle_message(context, *message),
            _ => println!("Unhandled message"),
        }
    }
}
