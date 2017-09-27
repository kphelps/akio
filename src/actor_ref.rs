use super::ActorCellHandle;

#[derive(Clone)]
pub struct ActorRef<T> {
    handle: ActorCellHandle<T>,
}

impl<T> ActorRef<T> {
    pub fn new(handle: ActorCellHandle<T>) -> Self {
        Self { handle: handle }
    }

    pub fn send(&self, message: T) {
        self.handle.enqueue_message(message)
    }
}
