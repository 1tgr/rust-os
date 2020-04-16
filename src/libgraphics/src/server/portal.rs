use crate::frame_buffer::FrameBuffer;
use crate::types::{Event, EventInput, Rect};
use crate::{client, ipc};
use alloc::sync::Arc;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use ecs::{ComponentStorage, System};
use os::{File, Mutex, Result, SharedMem};

const CURSOR_WIDTH: f64 = 32.0;
const CURSOR_HEIGHT: f64 = 32.0;
const CURSOR_HOTSPOT_X: f64 = 12.0;
const CURSOR_HOTSPOT_Y: f64 = 8.0;

#[derive(Clone)]
pub struct PortalRef {
    pub server2client: Arc<Mutex<File>>,
    pub portal_id: usize,
}

struct ScreenBuffer {
    pos: Rect,
    frame_buffer_ptr: *mut u8,
    portal_ref: PortalRef,
}

pub struct ScreenState {
    cursor_x: f64,
    cursor_y: f64,
    lfb: FrameBuffer,
    cursor_composite: FrameBuffer,
    cursor: CairoSurface<'static>,
    buffers: Vec<ScreenBuffer>,
}

unsafe impl Send for ScreenState {}

impl ScreenState {
    pub fn new(cursor_x: u16, cursor_y: u16, lfb: FrameBuffer) -> Self {
        static CURSOR_BYTES: &'static [u8] = include_bytes!("icons8-cursor-32.png");

        let cursor_composite = FrameBuffer::new(0.0, 0.0);
        let cursor = CairoSurface::from_png_slice(CURSOR_BYTES).unwrap();

        Self {
            cursor_x: cursor_x as f64 - CURSOR_HOTSPOT_X,
            cursor_y: cursor_y as f64 - CURSOR_HOTSPOT_Y,
            lfb,
            cursor_composite,
            cursor,
            buffers: Vec::new(),
        }
    }

    fn draw_buffers(cr: &Cairo, buffers: &[ScreenBuffer]) {
        cr.set_source_rgb(0.0, 0.0, 0.5).paint();

        for buffer in buffers {
            let ScreenBuffer {
                pos: Rect { x, y, width, height },
                frame_buffer_ptr,
                ..
            } = *buffer;
            let mut frame_buffer = unsafe { FrameBuffer::from_ptr(width, height, frame_buffer_ptr) };
            let surface = frame_buffer.as_surface();
            cr.set_source_surface(&surface, x, y).paint();
        }
    }

    fn find_portal(buffers: &[ScreenBuffer], x: f64, y: f64) -> Option<&PortalRef> {
        for buffer in buffers.iter().rev() {
            let ScreenBuffer {
                pos, ref portal_ref, ..
            } = *buffer;
            if x >= pos.x && y >= pos.y && x < pos.x + pos.width && y < pos.y + pos.height {
                return Some(portal_ref);
            }
        }

        None
    }

    fn cursor_composite_rect(&self) -> Rect {
        Rect {
            x: self.cursor_x - CURSOR_WIDTH,
            y: self.cursor_y - CURSOR_HEIGHT,
            width: CURSOR_WIDTH * 3.0 - CURSOR_HOTSPOT_X,
            height: CURSOR_HEIGHT * 3.0 - CURSOR_HOTSPOT_Y,
        }
    }

    fn draw_around_cursor(&mut self) {
        let cursor_composite_rect = self.cursor_composite_rect();

        let screen_rect = Rect {
            x: 0.0,
            y: 0.0,
            width: self.lfb.width,
            height: self.lfb.height,
        };

        {
            let cr = self.lfb.as_surface().into_cairo();

            cr.reset_clip()
                .new_path()
                .rectangle(
                    cursor_composite_rect.x,
                    cursor_composite_rect.y,
                    cursor_composite_rect.width,
                    cursor_composite_rect.height,
                )
                .new_sub_path()
                .move_to(screen_rect.x, screen_rect.y)
                .rel_line_to(0.0, screen_rect.height)
                .rel_line_to(screen_rect.width, 0.0)
                .rel_line_to(0.0, -screen_rect.height)
                .rel_line_to(-screen_rect.width, 0.0)
                .close_path()
                .clip();

            Self::draw_buffers(&cr, &self.buffers);
        }
    }

    fn draw_cursor(&mut self) {
        let cursor_composite_rect = self.cursor_composite_rect();
        self.cursor_composite
            .resize(cursor_composite_rect.width, cursor_composite_rect.height)
            .unwrap();

        {
            let cr = self.cursor_composite.as_surface().into_cairo();
            cr.save()
                .translate(-cursor_composite_rect.x, -cursor_composite_rect.y)
                .set_source_rgb(0.0, 0.0, 0.5)
                .paint();

            Self::draw_buffers(&cr, &self.buffers);

            cr.restore()
                .set_source_surface(&self.cursor, CURSOR_WIDTH, CURSOR_HEIGHT)
                .paint();
        }

        {
            let cursor_composite_surface = self.cursor_composite.as_surface();
            self.lfb
                .as_surface()
                .into_cairo()
                .set_source_surface(
                    &cursor_composite_surface,
                    cursor_composite_rect.x,
                    cursor_composite_rect.y,
                )
                .paint();
        }
    }

    pub fn update_mouse_state(&mut self, x: f64, y: f64, inputs: Vec<EventInput>) -> Result<()> {
        let prev_cursor_x = self.cursor_x;
        let prev_cursor_y = self.cursor_y;
        self.cursor_x = x as f64 - CURSOR_HOTSPOT_X;
        self.cursor_y = y as f64 - CURSOR_HOTSPOT_Y;

        let cr = self.lfb.as_surface().into_cairo();
        cr.rectangle(prev_cursor_x, prev_cursor_y, CURSOR_WIDTH, CURSOR_HEIGHT)
            .clip();

        Self::draw_buffers(&cr, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_x, self.cursor_y)
            .paint();

        if !inputs.is_empty() {
            if let Some(portal_ref) = Self::find_portal(&self.buffers, x, y) {
                let PortalRef {
                    ref server2client,
                    portal_id,
                } = *portal_ref;

                let mut server2client = server2client.lock().unwrap();
                for input in inputs {
                    let event = Event::Input { portal_id, input };
                    ipc::send_message(&mut *server2client, &event)?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct ZIndex {
    index: isize,
    version: usize,
}

impl ZIndex {
    pub fn new() -> Self {
        Self {
            index: 0,
            version: client::alloc_id(),
        }
    }
}

impl Clone for ZIndex {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            version: client::alloc_id(),
        }
    }
}

pub struct ServerPortal {
    portal_ref: PortalRef,
    pos: Rect,
    prev_pos: Rect,
    z_index: ZIndex,
    frame_buffer: FrameBuffer,
    needs_paint: bool,
}

impl ServerPortal {
    pub fn new(
        e: &ComponentStorage,
        server2client: Arc<Mutex<File>>,
        id: usize,
        pos: Rect,
        shared_mem: SharedMem,
    ) -> Result<Self> {
        let z_index = e
            .components::<Self>()
            .map(|p| &p.z_index)
            .max()
            .cloned()
            .unwrap_or_else(|| ZIndex::new());

        let frame_buffer = FrameBuffer::from_shared_mem(pos.width, pos.height, shared_mem)?;

        Ok(Self {
            portal_ref: PortalRef {
                server2client,
                portal_id: id,
            },
            pos,
            prev_pos: pos,
            z_index,
            frame_buffer,
            needs_paint: true,
        })
    }

    pub fn move_to(&mut self, pos: Rect) {
        self.pos = pos;
    }

    pub fn invalidate(&mut self) {
        self.needs_paint = true;
    }

    fn as_screen_buffer(&mut self) -> ScreenBuffer {
        ScreenBuffer {
            pos: self.pos,
            frame_buffer_ptr: self.frame_buffer.as_mut_slice().as_mut_ptr(),
            portal_ref: self.portal_ref.clone(),
        }
    }
}

pub struct ServerPortalSystem {
    screen_state: Arc<Mutex<ScreenState>>,
    input_state: Arc<Mutex<Option<PortalRef>>>,
}

impl ServerPortalSystem {
    pub fn new(screen_state: Arc<Mutex<ScreenState>>, input_state: Arc<Mutex<Option<PortalRef>>>) -> Self {
        ServerPortalSystem {
            screen_state,
            input_state,
        }
    }
}

impl System for ServerPortalSystem {
    fn run(&mut self, e: &mut ComponentStorage) -> Result<()> {
        let any_deleted = e.deleted_components::<ServerPortal>().any(|_| true);
        let mut portals = e.components_mut::<ServerPortal>().collect::<Vec<_>>();
        portals.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        for portal in portals.iter_mut() {
            if portal.prev_pos != portal.pos {
                portal.frame_buffer.resize(portal.pos.width, portal.pos.height)?;
                portal.prev_pos = portal.pos;
                portal.needs_paint = true;
            }
        }

        *self.input_state.lock().unwrap() = portals.last().map(|p| p.portal_ref.clone());

        if any_deleted || portals.iter().any(|p| p.needs_paint) {
            let mut screen = self.screen_state.lock().unwrap();
            screen.buffers.clear();

            for portal in portals.iter_mut() {
                screen.buffers.push(portal.as_screen_buffer());
                portal.needs_paint = false;
            }

            screen.draw_around_cursor();
            screen.draw_cursor();
        }

        Ok(())
    }
}
