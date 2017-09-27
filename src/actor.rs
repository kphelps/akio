use super::ActorCellHandle;

pub struct ActorRef<T> {
    handle: ActorCellHandle<T>,
}

impl<T> ActorRef<T> {
    pub fn new(handle: ActorCellHandle<T>) -> Self {
        Self { handle: handle }
    }

    pub fn send(&mut self, message: T) {
        self.handle.enqueue_message(message)
    }
}

pub trait Actor<T> {
    fn handle_message(&self, message: T);
}
