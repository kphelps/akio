use super::{ActorContext, ActorEvent, ActorRef, BaseActor, Mailbox, MailboxMessage};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use tokio_core::reactor::Remote;
use uuid::Uuid;

pub trait BaseActorCell {
    fn process_message(&mut self);
}

pub struct ActorCell {
    inner: Rc<RefCell<ActorCellInner>>,
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
    inner: Weak<RefCell<ActorCellInner>>,
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
        let p_inner = Rc::new(RefCell::new(inner));
        let handle = ActorCellHandle { inner: Rc::downgrade(&p_inner) };
        let actor_ref = ActorRef::new(handle);
        Self {
            inner: p_inner,
            actor: Box::new(actor),
            context: ActorContext::new(actor_ref, enqueuer, remote_handle),
        }
    }

    pub fn actor_ref(&self) -> ActorRef {
        self.context.self_ref.clone()
    }
}

impl ActorCellInner {
    pub fn enqueue_message(&mut self, message: Box<Any>, sender: ActorRef) {
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
}

impl BaseActorCell for ActorCell {
    fn process_message(&mut self) {
        let message = self.inner
            .borrow_mut()
            .next_to_process()
            .expect("mailbox is empty");
        match message {
            MailboxMessage::User(inner, sender) => {
                self.context.sender = sender;
                self.actor.handle_any(&self.context, inner);
            }
        };
    }
}

impl ActorCellHandle {
    pub fn enqueue_message(&self, message: Box<Any>, sender: ActorRef) {
        let inner = self.inner.upgrade().expect("Failed to enqueue");
        inner.borrow_mut().enqueue_message(message, sender);
    }
}
