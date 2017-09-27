use super::ActorContext;

pub trait Actor<T> {
    fn handle_message(&mut self, context: &ActorContext<T>, message: T);
}
