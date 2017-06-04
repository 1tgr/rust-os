use cairo::cairo::Cairo;
use graphics::{self,Event,FrameBuffer,Rect};
use os::{File,Mutex,Process,Result,SharedMem};
use std::sync::Arc;
use syscall::Handle;

struct WindowState {
    x: f64,
    y: f64,
    buffer: FrameBuffer,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowId {
    Desktop,
    Id(Handle, usize),
}

pub struct Window {
    id: WindowId,
    server2client: Arc<Mutex<File>>,
    state: Mutex<WindowState>,
}

impl Window {
    pub fn new(owner: &Process, id: usize, pos: Rect, shared_mem_handle: usize, server2client: Arc<Mutex<File>>) -> Result<Self> {
        let shared_mem = SharedMem::from_raw(owner.open_handle(shared_mem_handle)?, false);
        Ok(Window {
            id: WindowId::Id(owner.handle().get(), id),
            server2client,
            state: Mutex::new(WindowState {
                x: pos.x,
                y: pos.y,
                buffer: FrameBuffer::new(pos.width, pos.height, shared_mem)?,
            })?
        })
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn move_to(&self, pos: Rect) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.x = pos.x;
        state.y = pos.y;
        state.buffer.resize(pos.width, pos.height)?;
        Ok(())
    }

    pub fn paint_on(&self, cr: &Cairo) {
        let mut state = self.state.lock().unwrap();
        let (x, y) = (state.x, state.y);
        let surface = state.buffer.create_surface();
        cr.set_source_surface(&surface, x, y);
        cr.paint();
    }

    pub fn send_keypress(&self, c: char) -> Result<()> {
        match self.id {
            WindowId::Id(_, id) => {
                let mut server2client = self.server2client.lock().unwrap();
                graphics::send_message(&mut *server2client, Event::KeyPress { window_id: id, code: c })?;
            },

            _ => { }
        }

        Ok(())
    }
}
