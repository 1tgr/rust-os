use crate::pipe;
use alloc::collections::vec_deque::VecDeque;
use graphics_base::ipc;
use graphics_base::types::{Command, Event};
use graphics_base::Result;
use os::{File, OSHandle};

pub struct ClientPipe {
    buf: VecDeque<u8>,
    client2server: File,
    server2client: File,
}

impl ClientPipe {
    pub fn new() -> Self {
        Self {
            buf: VecDeque::new(),
            client2server: File::from_raw(OSHandle::from_raw(2)),
            server2client: File::from_raw(OSHandle::from_raw(3)),
        }
    }

    pub fn send_command(&mut self, command: &Command) -> Result<()> {
        ipc::send_message(&mut self.client2server, command)
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        ipc::read_message(&mut self.buf, &mut self.server2client)
    }

    pub fn checkpoint(&mut self) -> Result<usize> {
        let id = pipe::alloc_id();
        self.send_command(&Command::Checkpoint { id })?;
        Ok(id)
    }
}
