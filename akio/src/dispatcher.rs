#[cfg(target_os = "linux")]
use core_affinity;

#[cfg(not(target_os = "linux"))]
use num_cpus;

use super::{ActorCell, context};
use futures::future::Executor;
use futures::prelude::*;
use futures::sync::mpsc;
use rand;
use rand::Rng;
use std::iter;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio_core::reactor::{Core, Remote};


enum ThreadMessage {
    ProcessActor(ActorCell),
}

struct ThreadHandle {
    sender: mpsc::Sender<ThreadMessage>,
    handle: thread::JoinHandle<()>,
    remote: Remote,
}

impl ThreadHandle {
    pub fn join(self) {
        self.handle.join().expect("Shutdown failed")
    }
}

pub struct Dispatcher {
    handles: Vec<ThreadHandle>,
    thread_i: usize,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            thread_i: 0,
        }
    }

    pub fn start(&mut self) {
        self.handles = self.create_threads();
    }

    pub fn join(self) {
        self.handles.into_iter().for_each(ThreadHandle::join)
    }

    pub fn dispatch(&self, actor: ActorCell) {
        let thread_handle = self.next_thread();

        let f = thread_handle
            .sender
            .clone()
            .send(ThreadMessage::ProcessActor(actor))
            .map(|_| ())
            .map_err(|_| ());
        thread_handle.remote.execute(f).unwrap();
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
        let (remote, handle) = DispatcherThread::new(receiver).run_with_affinity(core_id);
        ThreadHandle {
            sender: sender,
            handle: handle,
            remote: remote,
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
        let (remote, handle) = DispatcherThread::new(receiver).run();
        ThreadHandle {
            sender: sender,
            handle: handle,
            remote: remote,
        }
    }

    fn next_thread(&self) -> &ThreadHandle {
        let mut rng = rand::thread_rng();
        let handle = rng.choose(&self.handles);
        handle.unwrap()
    }
}

struct DispatcherThread {
    receiver: mpsc::Receiver<ThreadMessage>,
}

impl DispatcherThread {
    pub fn new(receiver: mpsc::Receiver<ThreadMessage>) -> Self {
        Self { receiver: receiver }
    }

    pub fn run(self) -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            println!("Starting thread: {:?}", thread::current().id());
            let stream = self.receiver
                .for_each(|message| {
                              handle_message(message);
                              Ok(())
                          });
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext { handle: handle });
            core.run(stream).expect("Dispatcher failure");
        });
        // Need to extract the Remote from the new thread
        loop {
            if arc_remote.lock().unwrap().is_some() {
                break;
            }
            thread::sleep_ms(10);
        }
        let remote = arc_remote.lock().unwrap().as_ref().unwrap().clone();
        (remote, handle)
    }

    #[cfg(target_os = "linux")]
    pub fn run_with_affinity(self,
                             core_id: core_affinity::CoreId)
                             -> (Remote, thread::JoinHandle<()>) {
        let arc_remote = Arc::new(Mutex::new(None));
        let cloned_arc_remote = arc_remote.clone();
        let handle = thread::spawn(move || {
            println!("Starting thread: {:?}", thread::current().id());
            core_affinity::set_for_current(core_id);
            let stream = self.receiver
                .for_each(|message| {
                              handle_message(message);
                              Ok(())
                          });
            let mut core = Core::new().expect("Failed to start dispatcher thread");
            let handle = core.handle();
            *cloned_arc_remote.lock().unwrap() = Some(core.remote());
            context::set_thread_context(context::ThreadContext { handle: handle });
            core.run(stream).expect("Dispatcher failure");
        });
        // Need to extract the Remote from the new thread
        loop {
            if arc_remote.lock().unwrap().is_some() {
                break;
            }
            thread::sleep_ms(10);
        }
        let remote = arc_remote.lock().unwrap().as_ref().unwrap().clone();
        (remote, handle)
    }
}

fn handle_message(message: ThreadMessage) {
    match message {
        ThreadMessage::ProcessActor(mut actor_cell) => {
            actor_cell.process_messages(10);
            actor_cell.set_idle_or_dispatch();
        }
    }
}
