use super::Actor;
use futures::sync::oneshot;

struct AskActor<T> {
    promise: oneshot::Sender<T>,
}

impl<T> AskActor<T> {
    pub fn new(promise: oneshot::Sender<T>) -> Self {
        Self { promise: promise }
    }
}

impl<T> Actor for AskActor<T> {
    type Message = T;

    fn handle_message(&mut self, _context: &mut ActorContext, message: T) {
        match self.promise.send(message) {
            Some(_) => (),
            None => println!("Dead ask"),
        }
    }
}
