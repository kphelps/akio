use super::ActorRef;
use std::any::Any;

pub trait BaseActor: Send {
    fn handle_any(&mut self, message: Box<Any + Send>);

    fn on_start(&mut self);

    fn on_stop(&mut self);
}

pub trait Actor {
    type Message: 'static;

    fn handle_message(&mut self, message: Self::Message);

    fn on_start_impl(&mut self) {}

    fn on_stop_impl(&mut self) {}
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

    fn on_start(&mut self) {
        self.on_start_impl()
    }

    fn on_stop(&mut self) {
        self.on_stop_impl()
    }
}

pub trait TypedActor {
    type RefType;

    fn from_ref(actor_ref: &ActorRef) -> Self::RefType;
}
