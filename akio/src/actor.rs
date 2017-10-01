use super::ActorRef;
use std::any::Any;

pub trait BaseActor: Send {
    fn handle_any(&mut self, message: Box<Any + Send>);
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, message: Self::Message);
}

impl<T> BaseActor for T
    where T: Actor + Send
{
    fn handle_any(&mut self, any_message: Box<Any + Send>) {
        match any_message.downcast() {
            Ok(message) => self.handle_message(*message),
            _ => {
                println!("Unhandled message");
            }
        }
    }
}

pub trait TypedActor {
    type RefType;

    fn from_ref(actor_ref: &ActorRef) -> Self::RefType;
}
