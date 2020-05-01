use crate::frame_buffer::{AsSurface, AsSurfaceMut, FrameBuffer};
use crate::server::portal::PortalRef;
use crate::types::{EventInput, MouseButton, MouseInput, Rect};
use crate::Result;
use alloc::sync::Weak;
use cairo::bindings::{CAIRO_FORMAT_ARGB32, CAIRO_FORMAT_RGB24};
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;

const CURSOR_WIDTH: f64 = 32.0;
const CURSOR_HEIGHT: f64 = 32.0;
const CURSOR_HOTSPOT_X: f64 = 12.0;
const CURSOR_HOTSPOT_Y: f64 = 8.0;

pub struct ScreenBuffer {
    pub pos: Rect,
    pub frame_buffer_size: (u16, u16),
    pub frame_buffer: Weak<FrameBuffer>,
    pub portal_ref: PortalRef,
}

fn to_sprite(hotspot: (u16, u16)) -> (f64, f64) {
    (hotspot.0 as f64 - CURSOR_HOTSPOT_X, hotspot.1 as f64 - CURSOR_HOTSPOT_Y)
}

pub struct Screen<S> {
    cursor_hotspot: (u16, u16),
    cursor_sprite: (f64, f64),
    buttons: [bool; 3],
    screen_size: (u16, u16),
    lfb: S,
    cursor: CairoSurface<'static>,
    pub buffers: Vec<ScreenBuffer>,
}

unsafe impl<S> Send for Screen<S> {}

impl<S> Screen<S>
where
    S: AsSurfaceMut,
{
    pub fn new(screen_size: (u16, u16), lfb: S) -> Self {
        static CURSOR_BYTES: &'static [u8] = include_bytes!("icons8-cursor-32.png");

        let cursor = CairoSurface::from_png_slice(CURSOR_BYTES).unwrap();
        let cursor_hotspot = (screen_size.0 / 2, screen_size.1 / 2);

        Self {
            cursor_hotspot,
            cursor_sprite: to_sprite(cursor_hotspot),
            buttons: [false; 3],
            screen_size,
            lfb,
            cursor,
            buffers: Vec::new(),
        }
    }

    fn draw_buffers(cr: &Cairo, screen_size: (u16, u16), buffers: &[ScreenBuffer]) {
        cr.new_path()
            .move_to(0.0, 0.0)
            .rel_line_to(0.0, screen_size.1 as f64)
            .rel_line_to(screen_size.0 as f64, 0.0)
            .rel_line_to(0.0, -(screen_size.1 as f64))
            .rel_line_to(-(screen_size.0 as f64), 0.0);

        for buffer in buffers {
            let ScreenBuffer {
                pos: Rect { x, y, width, height },
                frame_buffer_size,
                ref frame_buffer,
                ..
            } = *buffer;

            if let Some(frame_buffer) = frame_buffer.upgrade() {
                let surface = frame_buffer.as_surface(CAIRO_FORMAT_RGB24, frame_buffer_size);
                cr.set_source_surface(&surface, x, y).paint();
            }

            cr.new_sub_path()
                .rectangle(x, y, width, height)
                .close_path()
                .clip_preserve();
        }

        cr.set_source_rgb(0.0, 0.0, 0.5).paint();
    }

    fn find_portal(buffers: &[ScreenBuffer], pos: (u16, u16)) -> Option<(f64, f64, &PortalRef)> {
        let x = pos.0 as f64;
        let y = pos.1 as f64;
        for buffer in buffers {
            let ScreenBuffer {
                pos, ref portal_ref, ..
            } = *buffer;

            if pos.contains(x, y) {
                return Some((x - pos.x, y - pos.y, portal_ref));
            }
        }

        None
    }

    #[cfg(target_os = "rust_os")]
    pub fn update_mouse_state_delta(&mut self, dx: i16, dy: i16, dw: i8, buttons: [bool; 3]) -> Result<()> {
        let x = ((self.cursor_hotspot.0 as i32 + dx as i32).max(0) as u16).min(self.screen_size.0 - 1);
        let y = ((self.cursor_hotspot.1 as i32 + dy as i32).max(0) as u16).min(self.screen_size.1 - 1);
        self.update_mouse_state(x, y, dw, buttons)
    }

    pub fn update_mouse_state(&mut self, x: u16, y: u16, _dw: i8, buttons: [bool; 3]) -> Result<()> {
        let prev_cursor_hotspot = self.cursor_hotspot;
        let prev_cursor_sprite = self.cursor_sprite;
        let prev_buttons = self.buttons;

        self.cursor_hotspot = (x, y);
        self.cursor_sprite = to_sprite(self.cursor_hotspot);
        self.buttons = buttons;

        if let Some((x, y, portal_ref)) = Self::find_portal(&self.buffers, self.cursor_hotspot) {
            let mut inputs = Vec::new();
            if prev_cursor_hotspot != self.cursor_hotspot {
                inputs.push(MouseInput::Move);
            }

            for ((&prev_down, &down), &button) in prev_buttons
                .iter()
                .zip(self.buttons.iter())
                .zip([MouseButton::Left, MouseButton::Middle, MouseButton::Right].iter())
            {
                if !prev_down && down {
                    inputs.push(MouseInput::ButtonDown { button });
                } else if prev_down && !down {
                    inputs.push(MouseInput::ButtonUp { button });
                }
            }

            for input in inputs {
                portal_ref.send_input(EventInput::Mouse { x, y, input })?;
            }
        }

        let cr = self
            .lfb
            .as_surface_mut(CAIRO_FORMAT_ARGB32, self.screen_size)
            .into_cairo();

        cr.rectangle(prev_cursor_sprite.0, prev_cursor_sprite.1, CURSOR_WIDTH, CURSOR_HEIGHT)
            .clip();

        Self::draw_buffers(&cr, self.screen_size, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_sprite.0, self.cursor_sprite.1)
            .paint();

        Ok(())
    }

    pub fn update_buffers<I>(&mut self, buffers: I)
    where
        I: IntoIterator<Item = ScreenBuffer>,
    {
        self.buffers.clear();
        self.buffers.extend(buffers);

        let cr = self
            .lfb
            .as_surface_mut(CAIRO_FORMAT_ARGB32, self.screen_size)
            .into_cairo();

        Self::draw_buffers(&cr, self.screen_size, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_sprite.0, self.cursor_sprite.1)
            .paint();
    }
}
