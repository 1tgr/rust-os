#![feature(collections)]
#![feature(link_args)]
#![feature(start)]

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
use collections::btree_map::BTreeMap;
use collections::vec_deque::VecDeque;
use graphics::{Command,Event,Rect};
use os::{File,Mutex,OSMem,Process,Result,Thread};
use std::io::Read;
use std::str;
use std::sync::Arc;
use syscall::libc_helpers;
use window::{Window,WindowId};

struct Server {
    lfb_mem: OSMem,
    windows_by_zorder: VecDeque<Arc<Window>>,
    windows_by_id: BTreeMap<WindowId, Arc<Window>>,
    focus_window: Option<Arc<Window>>,
}

fn ref_eq<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}

impl Server {
    pub fn new() -> Result<Self> {
        let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        Ok(Server {
            lfb_mem: unsafe { OSMem::from_raw(lfb_ptr, stride * 600) },
            windows_by_zorder: VecDeque::new(),
            windows_by_id: BTreeMap::new(),
            focus_window: None
        })
    }

    fn add_window(&mut self, window: Window) {
        let window = Arc::new(window);
        self.windows_by_zorder.push_front(window.clone());
        self.windows_by_id.insert(window.id(), window.clone());
        self.focus_window = Some(window);
        self.paint_all();
    }

    fn remove_window_impl(
        windows_by_zorder: &mut VecDeque<Arc<Window>>,
        windows_by_id: &mut BTreeMap<WindowId, Arc<Window>>,
        focus_window: &mut Option<Arc<Window>>,
        id: WindowId
    ) {
        if let Some(ref window) = windows_by_id.remove(&id) {
            let index_opt =
                windows_by_zorder
                    .iter()
                    .position(|w| ref_eq::<Window>(&*w, &*window));

            if let Some(index) = index_opt {
                windows_by_zorder.remove(index);
            }

            let action = {
                match *focus_window {
                    Some(ref old_focus_window) if ref_eq::<Window>(old_focus_window, window) => {
                        let next_window_opt =
                            index_opt
                                .and_then(|index| windows_by_zorder.get(index))
                                .or_else(|| windows_by_zorder.front())
                                .map(|w| (*w).clone());

                        Ok(next_window_opt)
                    },

                    _ => Err(()),
                }
            };

            if let Ok(next_window_opt) = action {
                *focus_window = next_window_opt;
            }
        }
    }

    fn remove_window(&mut self, id: WindowId) {
        Server::remove_window_impl(
            &mut self.windows_by_zorder,
            &mut self.windows_by_id,
            &mut self.focus_window,
            id);

        self.paint_all();
    }

    fn move_window(&mut self, id: WindowId, pos: Rect) -> Result<()> {
        if let Some(window) = self.windows_by_id.get_mut(&id) {
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

        let mut i = self.windows_by_zorder.iter_mut();
        while let Some(window) = i.next_back() {
            window.paint_on(&cr);
        }
    }

    pub fn handle_command(&mut self, client_process: &Process, server2client: &Arc<Mutex<File>>, command: Command) -> Result<()> {
        let handle = client_process.handle().get();
        println!("[Server] {:?}", command);
        match command {
            Command::Checkpoint { id } => {
                graphics::send_message(&mut *server2client.lock().unwrap(), Event::Checkpoint { id })?;
            },

            Command::CreateWindow { id, pos, shared_mem_handle } => {
                let window = Window::new(client_process, id, pos, shared_mem_handle, server2client.clone())?;
                self.add_window(window);
            },

            Command::DestroyWindow { id } => {
                let id = WindowId::Id(handle, id);
                self.remove_window(id);
            },

            Command::InvalidateWindow { id: _id } => {
                self.paint_all();
            },

            Command::MoveWindow { id, pos } => {
                let id = WindowId::Id(handle, id);
                self.move_window(id, pos)?;
            }
        }

        Ok(())
    }

    pub fn send_keypress(&mut self, c: char) -> Result<()> {
        if let Some(ref mut window) = self.focus_window {
            window.send_keypress(c)?;
        }

        Ok(())
    }
}

struct Connection<'a> {
    server: &'a Mutex<Server>,
    process: Process,
    client2server: File,
    server2client: Arc<Mutex<File>>,
}

impl<'a> Connection<'a> {
    pub fn new(server: &'a Mutex<Server>, filename: &str) -> Result<Self> {
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
            server2client: Arc::new(Mutex::new(server2client)?)
        })
    }

    pub fn run(mut self) -> Result<()> {
        let mut buf = VecDeque::new();
        loop {
            let c = graphics::read_message(&mut buf, &mut self.client2server)?;
            self.server.lock().unwrap().handle_command(&self.process, &self.server2client, c)?;
        }
    }
}

fn run() -> Result<()> {
    let server = Arc::new(Mutex::new(Server::new()?)?);

    {
        let server = server.clone();

        let run = move || -> Result<()> {
            let mut stdin = File::open("stdin")?;
            let mut buf = [0; 4];
            loop {
                let len = stdin.read(&mut buf)?;
                if let Ok(s) = str::from_utf8(&buf[..len]) {
                    if let Some(c) = s.chars().next() {
                        server.lock().unwrap().send_keypress(c)?;
                    }
                }
            }
        };

        Thread::spawn(move || {
            run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
        })?;
    }

    Connection::new(&*server, "graphics_client")?.run()
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
