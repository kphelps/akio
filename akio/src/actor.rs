use std::any::Any;
use super::ActorContext;

pub trait BaseActor: Send {
    fn handle_any(&mut self, context: &mut ActorContext, message: Box<Any + Send>);
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, context: &mut ActorContext, message: Self::Message);
}

impl<T> BaseActor for T
    where T: Actor + Send
{
    fn handle_any(&mut self, context: &mut ActorContext, any_message: Box<Any + Send>) {
        match any_message.downcast() {
            Ok(message) => self.handle_message(context, *message),
            _ => println!("Unhandled message"),
        }
    }
}
