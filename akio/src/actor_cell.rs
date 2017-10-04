use super::{
    context,
    create_actor,
    ActorChildren,
    ActorContext,
    ActorRef,
    ActorSystem,
    BaseActor,
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
pub struct ActorCellHandle {
    // ActorSystem holds the only RCs. When the actor is stopped the pointer
    // will fail to upgrade.
    cell: Weak<ActorCell>,
}

impl ActorCellHandle {
    pub fn new(p_cell: Weak<ActorCell>) -> Self {
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

    pub fn enqueue_message(&self, message: Box<Any + Send>, sender: ActorRef) {
        let me = self.clone();
        if let Err(e) = self.with_cell(|cell| cell.enqueue_message(me, message, sender)) {
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

    pub fn spawn<A>(&self, id: Uuid, actor: A) -> ActorRef
    where
        A: BaseActor + 'static,
    {
        self.with_cell_unwrapped(|cell| cell.spawn(id, actor))
    }

    pub fn with_children<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ActorChildren) -> R,
    {
        self.with_cell_unwrapped(|cell| f(&cell.children.lock()))
    }

    pub fn on_start(&self) {
        self.with_cell_unwrapped(|cell| cell.on_start())
    }

    fn with_cell<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(Arc<ActorCell>) -> R,
    {
        match self.cell.upgrade() {
            Some(cell) => Ok(f(cell)),
            None => bail!(ErrorKind::ActorDestroyed),
        }
    }

    fn with_cell_unwrapped<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Arc<ActorCell>) -> R,
    {
        self.with_cell(f).expect("actor ref invalidated")
    }
}

pub struct ActorCell {
    id: Uuid,
    mailbox: Mutex<Mailbox>,
    children: Mutex<ActorChildren>,
    status: Mutex<ActorStatus>,
    actor: Mutex<Box<BaseActor>>,
    system: ActorSystem,
}

impl ActorCell {
    pub fn new<A>(system: ActorSystem, id: Uuid, actor: A) -> Arc<ActorCell>
    where
        A: BaseActor + 'static,
    {
        let mailbox = Mutex::new(Mailbox::new());
        let cell = Self {
            id: id,
            mailbox: mailbox,
            children: Mutex::new(ActorChildren::new()),
            system: system,
            status: Mutex::new(ActorStatus::Idle),
            actor: Mutex::new(Box::new(actor)),
        };
        Arc::new(cell)
    }

    pub fn process_messages(&self, self_ref: ActorRef, max_count: usize) -> usize {
        let message_batch = self.next_batch_to_process(max_count);
        let count = message_batch.len();
        self.set_current_actor(self_ref);
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

    pub fn enqueue_message(&self, me: ActorCellHandle, message: Box<Any + Send>, sender: ActorRef) {
        self.mailbox.lock().push(message, sender);
        self.dispatch(me);
    }

    pub fn enqueue_system_message(&self, me: ActorCellHandle, message: SystemMessage) {
        self.mailbox.lock().push_system_message(message);
        self.dispatch(me);
    }

    pub fn set_idle_or_dispatch(&self, me: ActorCellHandle) {
        let mailbox = self.mailbox.lock();
        if mailbox.is_empty() {
            self.set_status(ActorStatus::Idle);
        } else {
            self.system.dispatch(me)
        }
    }

    pub fn spawn<A>(&self, id: Uuid, actor: A) -> ActorRef
    where
        A: BaseActor + 'static,
    {
        let actor_ref = create_actor(&self.system, id, actor);
        self.children.lock().insert(id, &actor_ref);
        actor_ref
    }

    fn set_current_actor(&self, self_ref: ActorRef) {
        context::set_current_actor(ActorContext::new(self_ref, self.system.clone()))
    }

    fn dispatch(&self, cell: ActorCellHandle) {
        if !self.status.lock().is_idle() {
            return;
        }
        self.set_status(ActorStatus::Scheduled);
        self.system.dispatch(cell);
    }

    fn process_message(&self, message: MailboxMessage) {
        match message {
            MailboxMessage::User(inner, sender) => {
                context::set_sender(sender);
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
                let children = self.children.lock().iter().cloned().collect::<Vec<ActorRef>>();
                let lock_fs = children.into_iter().map(|ar| ar.stop());
                let f = future::join_all(lock_fs)
                    .map(move |_| promise.send(()).unwrap());
                context::execute(f);
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
