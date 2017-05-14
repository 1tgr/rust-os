use ipc;
use os::{File,OSHandle};
use std::sync::atomic::{AtomicUsize,Ordering};
use syscall::Result;
use types::{Command,Event};

pub fn alloc_id() -> usize {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct Client {
    client2server: File,
    server2client: File,
}

impl Client {
    pub fn new() -> Self {
        Client {
            client2server: File::from_raw(OSHandle::from_raw(2)),
            server2client: File::from_raw(OSHandle::from_raw(3)),
        }
    }

    pub fn send_command(&mut self, command: Command) -> Result<()> {
        ipc::send_message(&mut self.client2server, command)
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        ipc::read_message(&mut self.server2client)
    }
}
