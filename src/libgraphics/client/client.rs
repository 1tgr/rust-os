use crate::ipc;
use crate::types::{Command, Event};
use alloc::collections::vec_deque::VecDeque;
use core::cell::RefCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use os::{File, OSHandle};
use syscall::Result;

pub fn alloc_id() -> usize {
    static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct Client {
    client2server: RefCell<File>,
    server2client: RefCell<(VecDeque<u8>, File)>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            client2server: RefCell::new(File::from_raw(OSHandle::from_raw(2))),
            server2client: RefCell::new((VecDeque::new(), File::from_raw(OSHandle::from_raw(3)))),
        }
    }

    pub fn send_command(&self, command: Command) -> Result<()> {
        let mut client2server = self.client2server.borrow_mut();
        ipc::send_message(&mut *client2server, command)
    }

    pub fn wait_for_event(&self) -> Result<Event> {
        let (ref mut buf, ref mut server2client) = *self.server2client.borrow_mut();
        ipc::read_message(buf, server2client)
    }

    pub fn checkpoint(&self) -> Result<usize> {
        let id = alloc_id();
        self.send_command(Command::Checkpoint { id })?;
        Ok(id)
    }
}
