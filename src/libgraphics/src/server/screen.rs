use crate::frame_buffer::FrameBuffer;
use crate::ipc;
use crate::types::{Event, EventInput, Rect};
use alloc::sync::Arc;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use os::{File, Mutex, Result};

const CURSOR_WIDTH: f64 = 32.0;
const CURSOR_HEIGHT: f64 = 32.0;
const CURSOR_HOTSPOT_X: f64 = 12.0;
const CURSOR_HOTSPOT_Y: f64 = 8.0;

#[derive(Clone)]
pub struct PortalRef {
    pub server2client: Arc<Mutex<File>>,
    pub portal_id: usize,
}

pub struct ScreenBuffer {
    pub pos: Rect,
    pub frame_buffer_ptr: *mut u8,
    pub portal_ref: PortalRef,
}

pub struct ScreenState {
    cursor_x: f64,
    cursor_y: f64,
    lfb: FrameBuffer,
    cursor: CairoSurface<'static>,
    pub buffers: Vec<ScreenBuffer>,
}

unsafe impl Send for ScreenState {}

impl ScreenState {
    pub fn new(cursor_x: u16, cursor_y: u16, lfb: FrameBuffer) -> Self {
        static CURSOR_BYTES: &'static [u8] = include_bytes!("icons8-cursor-32.png");

        let cursor = CairoSurface::from_png_slice(CURSOR_BYTES).unwrap();

        Self {
            cursor_x: cursor_x as f64 - CURSOR_HOTSPOT_X,
            cursor_y: cursor_y as f64 - CURSOR_HOTSPOT_Y,
            lfb,
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

    pub fn draw(&mut self) {
        let cr = self.lfb.as_surface().into_cairo();
        Self::draw_buffers(&cr, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_x, self.cursor_y)
            .paint();
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
