use crate::app::ServerApp;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use core::str;
use graphics_base::ipc;
use graphics_base::Result;
use os::{File, Mutex, Process};
use syscall::libc_helpers;

pub struct ServerPipe {
    server: ServerApp,
    process: Process,
    client2server: File,
    server2client: Arc<Mutex<File>>,
}

impl ServerPipe {
    pub fn new(server: ServerApp, filename: &str) -> Result<Self> {
        let (stdin, stdout) = unsafe { (libc_helpers::stdin, libc_helpers::stdout) };
        let client2server = File::create_pipe()?;
        let server2client = File::create_pipe()?;
        let inherit = [
            stdin,
            stdout,
            client2server.handle().get(),
            server2client.handle().get(),
        ];

        Ok(ServerPipe {
            server,
            process: Process::spawn(filename, &inherit)?,
            client2server,
            server2client: Arc::new(Mutex::new(server2client)?),
        })
    }

    pub fn run(mut self) -> Result<()> {
        let mut buf = VecDeque::new();
        loop {
            let c = ipc::read_message(&mut buf, &mut self.client2server)?;
            self.server.handle_command(&self.process, &self.server2client, c)?;
        }
    }
}
