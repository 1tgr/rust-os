use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use core::str;
use os::libc_helpers;
use os::{File, Mutex, Process};
use ui_types::ipc::read_message;
use ui_types::types::Command;
use ui_types::Result;

pub struct ServerPipe {
    process: Arc<Process>,
    client2server: File,
    server2client: Arc<Mutex<File>>,
}

impl ServerPipe {
    pub fn spawn(filename: &str) -> Result<Self> {
        let client2server = File::create_pipe();
        let server2client = File::create_pipe();
        let inherit = [
            libc_helpers::stdin,
            libc_helpers::stdout,
            client2server.handle().get(),
            server2client.handle().get(),
        ];

        let process = Process::spawn(filename, &inherit)?;

        Ok(ServerPipe {
            process: Arc::new(process),
            client2server,
            server2client: Arc::new(Mutex::new(server2client)),
        })
    }

    pub fn run<F>(mut self, mut f: F) -> Result<()>
    where
        F: FnMut(&Arc<Process>, &Arc<Mutex<File>>, Command) -> Result<()>,
    {
        let mut buf = VecDeque::new();
        loop {
            let c = read_message(&mut buf, &mut self.client2server)?;
            f(&self.process, &self.server2client, c)?;
        }
    }
}
