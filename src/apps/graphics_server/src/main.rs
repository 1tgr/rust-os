#![feature(link_args)]
#![feature(start)]
#![allow(dead_code)]

extern crate alloc;
extern crate alloc_system;
extern crate cairo;
extern crate core;
extern crate graphics;
extern crate os;
extern crate rt;
extern crate serde;
extern crate syscall;

mod window;

use crate::window::{Window, WindowId};
use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::rc::Rc;
use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use core::cell::RefCell;
use core::str;
use graphics::{Command, Event, Rect, WidgetTree};
use os::{File, OSMem, Process, Result, Thread};
use std::io::Read;
use syscall::libc_helpers;

struct ServerInput;

impl ServerInput {
    pub fn send_keypress(&self, _c: char) -> Result<()> {
        Ok(())
    }
}

struct Server {
    windows: WidgetTree<Window>,
    windows_by_id: BTreeMap<WindowId, Rc<Window>>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            windows: WidgetTree::new(),
            windows_by_id: BTreeMap::new(),
        }
    }

    fn add_window(&mut self, window: Window) {
        let window = Rc::new(window);
        self.windows_by_id.insert(window.id(), window.clone());
        self.windows.add(window);
    }

    fn remove_window(&mut self, id: WindowId) {
        if let Some(window) = self.windows_by_id.remove(&id) {
            self.windows.remove(&window);
        }
    }

    fn move_window(&mut self, id: WindowId, pos: Rect) -> Result<()> {
        if let Some(window) = self.windows_by_id.get_mut(&id) {
            self.windows.move_to(window, pos)?;
        }

        Ok(())
    }

    pub fn handle_command(
        &mut self,
        client_process: &Process,
        server2client: &Rc<RefCell<File>>,
        command: Command,
    ) -> Result<bool> {
        let handle = client_process.handle().get();
        println!("[Server] {:?}", command);
        match command {
            Command::Checkpoint { id } => {
                graphics::send_message(&mut *server2client.borrow_mut(), Event::Checkpoint { id })?;
            }

            Command::CreateWindow {
                id,
                pos,
                shared_mem_handle,
            } => {
                let window = Window::new(client_process, id, pos, shared_mem_handle, server2client.clone())?;
                self.add_window(window);
            }

            Command::DestroyWindow { id } => {
                let id = WindowId::Id(handle, id);
                self.remove_window(id);
            }

            Command::InvalidateWindow { id: _id } => {
                self.windows.set_paint_needed();
            }

            Command::MoveWindow { id, pos } => {
                let id = WindowId::Id(handle, id);
                self.move_window(id, pos)?;
            }
        }

        Ok(self.windows.get_paint_needed())
    }

    pub fn send_keypress(&mut self, c: char) -> Result<()> {
        if let Some(window) = self.windows.get_focus_mut() {
            window.send_keypress(c)?;
        }

        Ok(())
    }

    pub fn paint(&mut self, lfb_mem: &mut [u8]) {
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        let surface = CairoSurface::from_slice(lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride);
        let cr = Cairo::new(surface);
        self.windows.paint_on(&cr);
    }

    pub fn input(&self) -> ServerInput {
        ServerInput
    }
}

struct Connection {
    server: Server,
    process: Process,
    client2server: File,
    server2client: Rc<RefCell<File>>,
}

impl Connection {
    pub fn new(server: Server, filename: &str) -> Result<Self> {
        let (stdin, stdout) = unsafe { (libc_helpers::stdin, libc_helpers::stdout) };
        let client2server = File::create_pipe()?;
        let server2client = File::create_pipe()?;
        let inherit = [
            stdin,
            stdout,
            client2server.handle().get(),
            server2client.handle().get(),
        ];

        Ok(Connection {
            server,
            process: Process::spawn(filename, &inherit)?,
            client2server,
            server2client: Rc::new(RefCell::new(server2client)),
        })
    }

    pub fn run(mut self) -> Result<()> {
        let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        let mut lfb_mem = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };

        let mut buf = VecDeque::new();
        loop {
            let c = graphics::read_message(&mut buf, &mut self.client2server)?;
            if self.server.handle_command(&self.process, &self.server2client, c)? {
                self.server.paint(&mut lfb_mem);
            }
        }
    }
}

fn run() -> Result<()> {
    let server = Server::new();

    {
        let input = server.input();

        let run = move || -> Result<()> {
            let mut stdin = File::open("stdin")?;
            let mut buf = [0; 4];
            loop {
                let len = stdin.read(&mut buf)?;
                if let Ok(s) = str::from_utf8(&buf[..len]) {
                    if let Some(c) = s.chars().next() {
                        input.send_keypress(c)?;
                    }
                }
            }
        };

        Thread::spawn(move || run().map(|()| 0).unwrap_or_else(|num| -(num as i32)))?;
    }

    Connection::new(server, "graphics_client")?.run()
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
