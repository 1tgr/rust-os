use crate::pipe;
use alloc::boxed::Box;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;
use graphics_base::ipc;
use graphics_base::types::{Command, Event};
use graphics_base::Result;
use hecs::World;
use os::{File, Mutex, OSHandle};

pub type Callback = Box<dyn FnOnce(&mut World) -> Result<()> + Send>;

pub struct AppSync {
    callbacks: Arc<Mutex<Vec<Callback>>>,
    server2client: File,
}

impl AppSync {
    pub fn call<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) -> Result<()> + Send + 'static,
    {
        self.callbacks.lock().push(Box::new(f));
        let _ = ipc::send_message(&mut self.server2client, &Command::Checkpoint { id: 0 });
    }
}

pub struct ClientPipe {
    buf: VecDeque<u8>,
    client2server: File,
    server2client: File,
    callbacks: Arc<Mutex<Vec<Box<dyn FnOnce(&mut World) -> Result<()> + Send>>>>,
}

impl ClientPipe {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            client2server: File::from_raw(OSHandle::from_raw(2)),
            server2client: File::from_raw(OSHandle::from_raw(3)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn send_command(&mut self, command: &Command) -> Result<()> {
        ipc::send_message(&mut self.client2server, command)
    }

    pub fn wait_for_event(&mut self) -> Result<(Event, Vec<Callback>)> {
        let event = ipc::read_message(&mut self.buf, &mut self.server2client)?;
        let callbacks = mem::replace(&mut *self.callbacks.lock(), Vec::new());
        Ok((event, callbacks))
    }

    pub fn checkpoint(&mut self) -> Result<usize> {
        let id = pipe::alloc_id();
        self.send_command(&Command::Checkpoint { id })?;
        Ok(id)
    }

    pub fn sync(&self) -> AppSync {
        AppSync {
            callbacks: self.callbacks.clone(),
            server2client: self.server2client.duplicate().unwrap(),
        }
    }
}
