use super::{Actor, ActorContext, ActorEvent, ActorRef, Mailbox, MailboxMessage};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use tokio_core::reactor::Remote;
use uuid::Uuid;

pub trait BaseActorCell {
    fn process_message(&mut self);
}

pub struct ActorCell<T> {
    inner: Rc<RefCell<ActorCellInner<T>>>,
    context: ActorContext<T>,
    actor: Box<Actor<T>>,
}

pub struct ActorCellInner<T> {
    id: Uuid,
    mailbox: Mailbox<T>,
    enqueuer: mpsc::Sender<ActorEvent>,
    remote_handle: Remote,
}

#[derive(Clone)]
pub struct ActorCellHandle<T> {
    inner: Weak<RefCell<ActorCellInner<T>>>,
}

impl<T: Clone> ActorCell<T> {
    pub fn new<A>(id: Uuid,
                  actor: A,
                  enqueuer: mpsc::Sender<ActorEvent>,
                  remote_handle: Remote)
                  -> Self
        where A: Actor<T>,
              A: 'static
    {
        let inner = ActorCellInner::<T> {
            id: id,
            mailbox: Mailbox::new(),
            enqueuer: enqueuer.clone(),
            remote_handle: remote_handle.clone(),
        };
        let p_inner = Rc::new(RefCell::new(inner));
        let handle = ActorCellHandle::<T> { inner: Rc::downgrade(&p_inner) };
        let actor_ref = ActorRef::new(handle);
        Self {
            inner: p_inner,
            actor: Box::new(actor),
            context: ActorContext::new(actor_ref, enqueuer, remote_handle),
        }
    }

    pub fn actor_ref(&self) -> ActorRef<T> {
        self.context.self_ref.clone()
    }
}

impl<T> ActorCellInner<T> {
    pub fn enqueue_message(&mut self, message: T) {
        self.mailbox.push(message);
        let f = self.enqueuer
            .clone()
            .send(ActorEvent::MailboxReady(self.id))
            .map(|_| ())
            .map_err(|_| ());
        self.remote_handle.execute(f).expect("readying mailbox");
    }

    pub fn next_to_process(&mut self) -> Option<MailboxMessage<T>> {
        self.mailbox.pop()
    }
}

impl<T: Clone> BaseActorCell for ActorCell<T> {
    fn process_message(&mut self) {
        let message = self.inner
            .borrow_mut()
            .next_to_process()
            .expect("mailbox is empty");
        match message {
            MailboxMessage::User(inner) => self.actor.handle_message(&self.context, inner),
        }
    }
}

impl<T> ActorCellHandle<T> {
    pub fn enqueue_message(&self, message: T) {
        let inner = self.inner.upgrade().expect("Failed to enqueue");
        inner.borrow_mut().enqueue_message(message);
    }
}
