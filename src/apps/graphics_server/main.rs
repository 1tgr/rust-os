#![feature(collections)]
#![feature(link_args)]
#![feature(start)]

#[macro_use]
extern crate intrusive_collections;

extern crate cairo;
extern crate collections;
extern crate graphics;
extern crate os;
extern crate serde;
extern crate syscall;

mod window;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use collections::vec_deque::VecDeque;
use graphics::{Command,Rect};
use intrusive_collections::{LinkedList,RBTree};
use os::{File,Mutex,OSMem,Process,Result};
use std::rc::Rc;
use syscall::libc_helpers;
use window::{Window,WindowId,WindowZOrderAdapter,WindowIdAdapter};

struct ServerState {
    lfb_mem: OSMem,
    windows_by_zorder: LinkedList<WindowZOrderAdapter>,
    windows_by_id: RBTree<WindowIdAdapter>,
}

impl ServerState {
    fn add_window(&mut self, window: Window) {
        let window = Rc::new(window);
        self.windows_by_zorder.push_back(window.clone());
        self.windows_by_id.insert(window);
        self.paint_all();
    }

    fn remove_window(&mut self, id: WindowId) {
        {
            let mut cursor = self.windows_by_zorder.front_mut();
            while let Some(window) = cursor.get() {
                if window.id() == id {
                    cursor.remove();
                }

                cursor.move_next();
            }
        }

        self.windows_by_id.find_mut(&id).remove();
        self.paint_all();
    }

    fn move_window(&mut self, id: WindowId, pos: Rect) -> Result<()> {
        if let Some(window) = self.windows_by_id.find_mut(&id).get() {
            window.move_to(pos)?;
        }

        self.paint_all();
        Ok(())
    }

    fn create_surface(&mut self) -> CairoSurface {
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        CairoSurface::from_slice(&mut self.lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride)
    }

    fn paint_all(&mut self) {
        let cr = Cairo::new(self.create_surface());
        cr.set_source_rgb(0.0, 0.0, 0.5);
        cr.paint();

        for window in self.windows_by_zorder.iter() {
            window.paint_on(&cr);
        }
    }
}

struct Server {
    state: Mutex<ServerState>,
}

impl Server {
    pub fn new() -> Result<Self> {
        let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        Ok(Server {
            state: Mutex::new(ServerState {
                lfb_mem: unsafe { OSMem::from_raw(lfb_ptr, stride * 600) },
                windows_by_zorder: LinkedList::new(WindowZOrderAdapter::new()),
                windows_by_id: RBTree::new(WindowIdAdapter::new()),
            })?,
        })
    }

    pub fn handle_command(&self, client_process: &Process, command: Command) -> Result<()> {
        let handle = client_process.handle().get();
        println!("[Server] {:?}", command);
        match command {
            Command::CreateWindow { id, pos, shared_mem_handle } => {
                let window = Window::new(client_process, id, pos, shared_mem_handle)?;
                self.state.lock()?.add_window(window);
            },

            Command::DestroyWindow { id } => {
                let id = WindowId::Id(handle, id);
                self.state.lock()?.remove_window(id);
            },

            Command::InvalidateWindow { id: _id } => {
                self.state.lock()?.paint_all();
            },

            Command::MoveWindow { id, pos } => {
                let id = WindowId::Id(handle, id);
                self.state.lock()?.move_window(id, pos)?;
            }
        }

        Ok(())
    }
}

struct Connection<'a> {
    server: &'a Server,
    process: Process,
    client2server: File,
    server2client: File,
}

impl<'a> Connection<'a> {
    pub fn new(server: &'a Server, filename: &str) -> Result<Self> {
        let (stdin, stdout) = unsafe { (libc_helpers::stdin, libc_helpers::stdout) };
        let client2server = File::create_pipe()?;
        let server2client = File::create_pipe()?;
        let inherit = [
            stdin,
            stdout,
            client2server.handle().get(),
            server2client.handle().get(),
        ];

        let process = Process::spawn(filename, &inherit)?;
        Ok(Connection { server, process, client2server, server2client })
    }

    pub fn run(mut self) -> Result<()> {
        let mut buf = VecDeque::new();
        loop {
            let c = graphics::read_message(&mut buf, &mut self.client2server)?;
            self.server.handle_command(&self.process, c)?;
        }
    }
}

fn run() -> Result<()> {
    let server = Server::new()?;
    Connection::new(&server, "graphics_client")?.run()
}

#[cfg(target_arch="x86_64")]
#[link_args = "-T ../../libsyscall/arch/amd64/link.ld"]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
