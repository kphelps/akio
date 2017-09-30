#[cfg(target_os = "linux")]
use core_affinity;

#[cfg(not(target_os = "linux"))]
use num_cpus;

use super::{ActorCell, ActorEvent};
use futures::prelude::*;
use futures::sync::mpsc;
use std::iter;
use std::thread;
use tokio_core::reactor::Core;


enum ThreadMessage {
    ProcessActor(ActorCell),
}

struct ThreadHandle {
    sender: mpsc::Sender<ThreadMessage>,
    handle: thread::JoinHandle<()>,
}

impl ThreadHandle {
    pub fn join(self) {
        self.handle.join().expect("Shutdown failed")
    }
}

pub struct Dispatcher {
    handles: Vec<ThreadHandle>,
    thread_i: usize,
    to_system: mpsc::Sender<ActorEvent>,
}

impl Dispatcher {
    pub fn new(to_system: mpsc::Sender<ActorEvent>) -> Self {
        Self {
            handles: Vec::new(),
            thread_i: 0,
            to_system: to_system,
        }
    }

    pub fn start(&mut self) {
        self.handles = self.create_threads();
    }

    pub fn join(self) {
        self.handles.into_iter().for_each(ThreadHandle::join)
    }

    pub fn dispatch(&mut self, actor: ActorCell) -> Box<Future<Item = (), Error = ()>> {
        let thread_handle = self.next_thread();
        Box::new(thread_handle
                     .sender
                     .clone()
                     .send(ThreadMessage::ProcessActor(actor))
                     .map(|_| ())
                     .map_err(|_| ()))
    }

    #[cfg(target_os = "linux")]
    fn create_threads(&self) -> Vec<ThreadHandle> {
        core_affinity::get_core_ids()
            .unwrap()
            .into_iter()
            .map(|core_id| self.create_thread(core_id))
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(target_os = "linux")]
    fn create_thread(&self, core_id: core_affinity::CoreId) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let to_system = self.to_system.clone();
        ThreadHandle {
            sender: sender,
            handle: thread::spawn(move || {
                                      core_affinity::set_for_current(core_id);
                                      DispatcherThread::new(receiver, to_system).run()
                                  }),
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn create_threads(&self) -> Vec<ThreadHandle> {
        iter::repeat(())
            .take(num_cpus::get())
            .map(|_| self.create_thread())
            .collect::<Vec<ThreadHandle>>()
    }

    #[cfg(not(target_os = "linux"))]
    fn create_thread(&self) -> ThreadHandle {
        let (sender, receiver) = mpsc::channel(100);
        let to_system = self.to_system.clone();
        ThreadHandle {
            sender: sender,
            handle: thread::spawn(move || DispatcherThread::new(receiver, to_system).run()),
        }
    }

    fn next_thread(&mut self) -> &ThreadHandle {
        let handle = &self.handles[self.thread_i];
        self.thread_i += 1;
        self.thread_i %= self.handles.len();
        handle
    }
}

struct DispatcherThread {
    receiver: mpsc::Receiver<ThreadMessage>,
    to_system: mpsc::Sender<ActorEvent>,
}

impl DispatcherThread {
    pub fn new(receiver: mpsc::Receiver<ThreadMessage>,
               to_system: mpsc::Sender<ActorEvent>)
               -> Self {
        Self {
            receiver: receiver,
            to_system: to_system,
        }
    }

    pub fn run(self) {
        println!("Starting thread: {:?}", thread::current().id());
        let to_system = self.to_system;
        let stream = self.receiver
            .for_each(|message| handle_message(to_system.clone(), message));
        Core::new()
            .expect("Failed to start dispatcher thread")
            .run(stream)
            .expect("Dispatcher failure");
    }
}

fn handle_message(to_system: mpsc::Sender<ActorEvent>,
                  message: ThreadMessage)
                  -> Box<Future<Item = (), Error = ()>> {
    match message {
        ThreadMessage::ProcessActor(mut actor_cell) => {
            Box::new(actor_cell
                         .process_messages(10)
                         .and_then(|_| {
                                       to_system
                                           .send(ActorEvent::ActorIdle(actor_cell))
                                           .map(|_| ())
                                           .map_err(|_| ())
                                   }))
        }
    }
}
