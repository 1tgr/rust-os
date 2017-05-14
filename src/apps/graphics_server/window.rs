use cairo::cairo::Cairo;
use graphics::{FrameBuffer,Rect};
use intrusive_collections::{KeyAdapter,LinkedListLink,RBTreeLink};
use os::{Process,Result,SharedMem};
use std::cell::RefCell;
use std::rc::Rc;
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
    by_zorder: LinkedListLink,
    by_id: RBTreeLink,
    id: WindowId,
    state: RefCell<WindowState>,
}

intrusive_adapter!(pub WindowZOrderAdapter = Rc<Window>: Window { by_zorder: LinkedListLink });
intrusive_adapter!(pub WindowIdAdapter = Rc<Window>: Window { by_id: RBTreeLink });

impl<'a> KeyAdapter<'a> for WindowIdAdapter {
    type Key = WindowId;
    fn get_key(&self, x: &'a Window) -> WindowId { x.id() }
}

impl Window {
    pub fn new(owner: &Process, id: usize, pos: Rect, shared_mem_handle: usize) -> Result<Self> {
        let shared_mem = SharedMem::from_raw(owner.open_handle(shared_mem_handle)?, false);
        Ok(Window {
            by_zorder: LinkedListLink::new(),
            by_id: RBTreeLink::new(),
            id: WindowId::Id(owner.handle().get(), id),
            state: RefCell::new(WindowState {
                x: pos.x,
                y: pos.y,
                buffer: FrameBuffer::new(pos.width, pos.height, shared_mem)?,
            })
        })
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn move_to(&self, pos: Rect) -> Result<()> {
        let mut state = self.state.borrow_mut();
        state.x = pos.x;
        state.y = pos.y;
        state.buffer.resize(pos.width, pos.height)?;
        Ok(())
    }

    pub fn paint_on(&self, cr: &Cairo) {
        let mut state = self.state.borrow_mut();
        let (x, y) = (state.x, state.y);
        let surface = state.buffer.create_surface();
        cr.set_source_surface(&surface, x, y);
        cr.paint();
    }
}
