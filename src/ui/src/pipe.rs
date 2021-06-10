use alloc::collections::vec_deque::VecDeque;
use os::{File, OSHandle};
use ui_types::ipc::{read_message, send_message};
use ui_types::types::{Command, Event};
use ui_types::Result;

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
        send_message(&mut self.client2server, command)
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        read_message(&mut self.buf, &mut self.server2client)
    }
}
