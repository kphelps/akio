use super::{ActorContext, ActorEvent, ActorRef, BaseActor, Mailbox, MailboxMessage};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use std::any::Any;
use std::sync::{Arc, Mutex};
use tokio_core::reactor::Remote;
use uuid::Uuid;

pub struct ActorCell {
    inner: Arc<Mutex<ActorCellInner>>,
    context: ActorContext,
    actor: Box<BaseActor>,
}

pub struct ActorCellInner {
    id: Uuid,
    mailbox: Mailbox,
    enqueuer: mpsc::Sender<ActorEvent>,
    remote_handle: Remote,
}

#[derive(Clone)]
pub struct ActorCellHandle {
    inner: Arc<Mutex<ActorCellInner>>,
}

impl ActorCell {
    pub fn new<A>(id: Uuid,
                  actor: A,
                  enqueuer: mpsc::Sender<ActorEvent>,
                  remote_handle: Remote)
                  -> Self
        where A: BaseActor + 'static
    {
        let inner = ActorCellInner {
            id: id,
            mailbox: Mailbox::new(),
            enqueuer: enqueuer.clone(),
            remote_handle: remote_handle.clone(),
        };
        let p_inner = Arc::new(Mutex::new(inner));
        let handle = ActorCellHandle { inner: p_inner.clone() };
        let actor_ref = ActorRef::new(handle);
        Self {
            inner: p_inner,
            actor: Box::new(actor),
            context: ActorContext::new(actor_ref, enqueuer, remote_handle),
        }
    }

    pub fn id(&self) -> Uuid {
        self.inner.lock().unwrap().id.clone()
    }

    pub fn actor_ref(&self) -> ActorRef {
        self.context.self_ref.clone()
    }

    fn process_message(&mut self, message: MailboxMessage) {
        match message {
            MailboxMessage::User(inner, sender) => {
                self.context.sender = sender;
                self.actor.handle_any(&mut self.context, inner);
            }
        };
    }

    pub fn process_messages(&mut self, max_count: usize) {
        let messages = self.inner.lock().unwrap().next_batch_to_process(max_count);
        messages
            .into_iter()
            .for_each(|message| self.process_message(message));
    }
}

impl ActorCellInner {
    pub fn enqueue_message(&mut self, message: Box<Any + Send>, sender: ActorRef) {
        self.mailbox.push(message, sender);
        let f = self.enqueuer
            .clone()
            .send(ActorEvent::MailboxReady(self.id))
            .map(|_| ())
            .map_err(|_| ());
        self.remote_handle.execute(f).expect("readying mailbox");
    }

    pub fn next_to_process(&mut self) -> Option<MailboxMessage> {
        self.mailbox.pop()
    }

    pub fn next_batch_to_process(&mut self, count: usize) -> Vec<MailboxMessage> {
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            match self.next_to_process() {
                Some(message) => v.push(message),
                None => return v,
            }
        }
        v
    }
}

impl ActorCellHandle {
    pub fn enqueue_message(&self, message: Box<Any + Send>, sender: ActorRef) {
        self.inner.lock().unwrap().enqueue_message(message, sender);
    }
}
