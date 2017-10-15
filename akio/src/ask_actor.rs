use super::{Actor, MessageHandler};
use futures::sync::oneshot;

pub struct AskActor<T> {
    promise: Option<oneshot::Sender<T>>,
}

impl<T> AskActor<T> {
    pub fn new(promise: oneshot::Sender<T>) -> Self {
        Self {
            promise: Some(promise),
        }
    }
}

impl<T> Actor for AskActor<T> {}

impl<T> MessageHandler<T> for AskActor<T> {
    fn handle(&mut self, message: T) {
        let promise = ::std::mem::replace(&mut self.promise, None);
        match promise.unwrap().send(message) {
            Ok(_) => (),
            Err(_) => println!("Dead ask"),
        }
    }
}
