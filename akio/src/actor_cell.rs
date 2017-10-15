use super::{
    context,
    create_actor,
    Actor,
    ActorRef,
    ActorSystem,
    Mailbox,
    MailboxMessage,
    SystemMessage,
};
use super::errors::*;
use futures::future;
use futures::prelude::*;
use parking_lot::{Mutex};
use std::any::Any;
use std::collections::VecDeque;
use std::sync::{Arc, Weak};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActorStatus {
    Idle,
    Scheduled,
    Suspended,
    Terminated,
}

impl ActorStatus {
    pub fn is_scheduled(&self) -> bool {
        *self == ActorStatus::Scheduled
    }

    pub fn is_idle(&self) -> bool {
        *self == ActorStatus::Idle
    }

    pub fn is_suspended(&self) -> bool {
        *self == ActorStatus::Suspended
    }

    pub fn is_terminated(&self) -> bool {
        *self == ActorStatus::Terminated
    }
}

#[derive(Clone)]
pub struct ActorCellHandle<A> {
    // ActorSystem holds the only RCs. When the actor is stopped the pointer
    // will fail to upgrade.
    cell: Weak<ActorCell<A>>,
}

impl<A> ActorCellHandle<A>
    where A: Actor
{
    pub fn new(p_cell: Weak<ActorCell<A>>) -> Self {
        Self {
            cell: p_cell,
        }
    }

    pub fn exists(&self) -> bool {
        self.cell.upgrade().is_some()
    }

    pub fn id(&self) -> Uuid {
        self.with_cell_unwrapped(|cell| cell.id.clone())
    }

    pub fn process_messages(&self, max_count: usize) -> usize {
        let self_ref = ActorRef::new(self.clone());
        self.with_cell_unwrapped(|cell| cell.process_messages(self_ref, max_count))
    }

    pub fn enqueue_message(&self, message: Box<Any + Send>)
        where B: Actor
    {
        let me = self.clone();
        if let Err(e) = self.with_cell(|cell| cell.enqueue_message(me, message)) {
            println!("Enqueued dead message: {}", e);
        }
    }

    pub fn enqueue_system_message(&self, message: SystemMessage) {
        let me = self.clone();
        if let Err(e) = self.with_cell(|cell| cell.enqueue_system_message(me, message)) {
            println!("Enqueued dead system message: {}", e);
        }
    }

    pub fn set_idle_or_dispatch(&self) {
        let me = self.clone();
        let _ = self.with_cell(|cell| cell.set_idle_or_dispatch(me));
    }

    pub fn spawn<B>(&self, id: Uuid, actor: B) -> ActorRef<B>
    where
        B: Actor,
    {
        self.with_cell_unwrapped(|cell| cell.spawn(id, actor))
    }

    pub fn on_start(&self) {
        self.with_cell_unwrapped(|cell| cell.on_start())
    }

    fn with_cell<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(Arc<ActorCell<A>>) -> R,
    {
        match self.cell.upgrade() {
            Some(cell) => Ok(f(cell)),
            None => bail!(ErrorKind::ActorDestroyed),
        }
    }

    fn with_cell_unwrapped<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Arc<ActorCell<A>>) -> R,
    {
        self.with_cell(f).expect("actor ref invalidated")
    }
}

pub struct ActorCell<A> {
    id: Uuid,
    mailbox: Mutex<Mailbox>,
    status: Mutex<ActorStatus>,
    actor: Mutex<A>,
    system: ActorSystem,
}

impl<A> ActorCell<Actor> {
    pub fn new(system: ActorSystem, id: Uuid, actor: A) -> Arc<ActorCell<A>>
    {
        let mailbox = Mutex::new(Mailbox::new());
        let cell = Self {
            id: id,
            mailbox: mailbox,
            system: system,
            status: Mutex::new(ActorStatus::Idle),
            actor: Mutex::new(Box::new(actor)),
        };
        Arc::new(cell)
    }

    pub fn process_messages(&self, self_ref: ActorRef<A>, max_count: usize) -> usize {
        let message_batch = self.next_batch_to_process(max_count);
        let count = message_batch.len();
        message_batch
            .into_iter()
            .for_each(|message| self.process_message(message));
        count
    }

    pub fn next_batch_to_process(&self, count: usize) -> VecDeque<MailboxMessage> {
        let mut mailbox = self.mailbox.lock();
        let mut v = VecDeque::with_capacity(count);
        for _ in 0..count {
            match mailbox.pop() {
                Some(message) => v.push_back(message),
                None => return v,
            }
        }
        v
    }

    pub fn enqueue_message(&self, me: ActorCellHandle<A>, message: Box<Any + Send>)
        where B: Actor
    {
        self.mailbox.lock().push(message);
        self.dispatch(me);
    }

    pub fn enqueue_system_message(&self, me: ActorCellHandle<A>, message: SystemMessage) {
        self.mailbox.lock().push_system_message(message);
        self.dispatch(me);
    }

    pub fn set_idle_or_dispatch(&self, me: ActorCellHandle<A>) {
        let mailbox = self.mailbox.lock();
        if mailbox.is_empty() {
            self.set_status(ActorStatus::Idle);
        } else {
            self.system.dispatch(me)
        }
    }

    pub fn spawn<B>(&self, id: Uuid, actor: B) -> ActorRef<B>
    where
        B: Actor,
    {
        create_actor(&self.system, id, actor)
    }

    fn dispatch(&self, cell: ActorCellHandle<A>) {
        if !self.status.lock().is_idle() {
            return;
        }
        self.set_status(ActorStatus::Scheduled);
        self.system.dispatch(cell);
    }

    fn process_message(&self, message: MailboxMessage) {
        match message {
            MailboxMessage::User(inner) => {
                self.actor.lock().handle_any(inner)
            }
            MailboxMessage::System(inner) => self.handle_system_message(inner),
        }
    }

    fn handle_system_message(&self, system_message: SystemMessage) {
        match system_message {
            SystemMessage::Stop(promise) => {
                self.set_status(ActorStatus::Terminated);
                self.actor.lock().on_stop();
                let _ = self.system.deregister_actor(&self.id);
            }
        }
    }

    pub fn on_start(&self) {
        self.actor.lock().on_start();
    }

    pub fn set_status(&self, status: ActorStatus) {
        *self.status.lock() = status;
    }
}
