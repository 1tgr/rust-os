use crate::portal::PortalRef;
use alloc::sync::Weak;
use cairo::bindings::{CAIRO_FORMAT_A8, CAIRO_FORMAT_ARGB32, CAIRO_FORMAT_RGB24};
use cairo::{Cairo, Surface};
use core::mem;
use graphics_base::frame_buffer::{AsSurface, AsSurfaceMut, FrameBuffer};
use graphics_base::types::{EventInput, MouseButton, MouseInputInfo, Rect};
use graphics_base::{Error, Result};
use jpeg_decoder::{Decoder, ImageInfo, PixelFormat};

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

pub struct InputCapture {
    button: MouseButton,
    pub pos: Rect,
    pub portal_ref: PortalRef,
}

fn to_sprite(hotspot: (u16, u16)) -> (f64, f64) {
    (hotspot.0 as f64 - CURSOR_HOTSPOT_X, hotspot.1 as f64 - CURSOR_HOTSPOT_Y)
}

fn surface_from_jpeg_slice(data: &[u8]) -> Result<Surface<'static>> {
    let mut decoder = Decoder::new(data);
    let data = decoder.decode().map_err(|_| Error::NotSupported)?;

    let ImageInfo {
        width,
        height,
        pixel_format,
    } = decoder.info().unwrap();

    let (data, format) = match pixel_format {
        PixelFormat::L8 => (data, CAIRO_FORMAT_A8),
        PixelFormat::RGB24 => {
            let mut data32 = Vec::with_capacity(width as usize * height as usize * 4);
            for chunk in data.chunks_exact(3) {
                data32.extend(chunk.iter().rev());
                data32.push(0);
            }

            (data32, CAIRO_FORMAT_RGB24)
        }
        PixelFormat::CMYK32 => panic!("CMYK not supported"),
    };

    Ok(Surface::from_vec(data, format, width, height))
}

pub struct Screen<S> {
    cursor_hotspot: (u16, u16),
    cursor_sprite: (f64, f64),
    buttons: [bool; 3],
    screen_size: (u16, u16),
    lfb: S,
    cursor: Surface<'static>,
    wallpaper: Surface<'static>,
    pub buffers: Vec<ScreenBuffer>,
    pub input_capture: Option<InputCapture>,
}

unsafe impl<S> Send for Screen<S> {}

impl<S> Screen<S>
where
    S: AsSurfaceMut,
{
    pub fn new(screen_size: (u16, u16), lfb: S) -> Self {
        static CURSOR_BYTES: &'static [u8] = include_bytes!("icons8-cursor-32.png");
        static WALLPAPER_BYTES: &'static [u8] = include_bytes!("wallpaper.jpg");

        let cursor = Surface::from_png_slice(CURSOR_BYTES).unwrap();
        let cursor_hotspot = (screen_size.0 / 2, screen_size.1 / 2);
        let wallpaper = surface_from_jpeg_slice(WALLPAPER_BYTES).unwrap();

        Self {
            cursor_hotspot,
            cursor_sprite: to_sprite(cursor_hotspot),
            buttons: [false; 3],
            screen_size,
            lfb,
            cursor,
            wallpaper,
            buffers: Vec::new(),
            input_capture: None,
        }
    }

    fn draw_buffers(cr: &Cairo, screen_size: (u16, u16), wallpaper: &Surface, buffers: &[ScreenBuffer]) {
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

        cr.set_source_surface(wallpaper, 0.0, 0.0).paint();
    }

    fn find_portal(&self) -> Option<(Rect, PortalRef)> {
        let pos = self.cursor_hotspot;
        let x = pos.0 as f64;
        let y = pos.1 as f64;
        if let Some(InputCapture {
            pos, ref portal_ref, ..
        }) = self.input_capture
        {
            Some((pos, portal_ref.clone()))
        } else {
            for buffer in self.buffers.iter() {
                let ScreenBuffer {
                    pos, ref portal_ref, ..
                } = *buffer;

                if pos.contains(x, y) {
                    return Some((pos, portal_ref.clone()));
                }
            }

            None
        }
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

        if let Some((pos, portal_ref)) = self.find_portal() {
            let screen_x = x as f64;
            let screen_y = y as f64;
            let x = screen_x - pos.x;
            let y = screen_y - pos.y;
            let info = MouseInputInfo {
                x,
                y,
                screen_x,
                screen_y,
            };

            let mut inputs = Vec::new();
            if prev_cursor_hotspot != self.cursor_hotspot {
                inputs.push(EventInput::MouseMove { info: info.clone() });
            }

            for ((&prev_down, &down), &button) in prev_buttons
                .iter()
                .zip(self.buttons.iter())
                .zip([MouseButton::Left, MouseButton::Middle, MouseButton::Right].iter())
            {
                if !prev_down && down {
                    inputs.push(EventInput::MouseButtonDown {
                        info: info.clone(),
                        button,
                    });

                    if self.input_capture.is_none() {
                        self.input_capture = Some(InputCapture {
                            button,
                            pos,
                            portal_ref: portal_ref.clone(),
                        });
                    }
                } else if prev_down && !down {
                    inputs.push(EventInput::MouseButtonUp {
                        info: info.clone(),
                        button,
                    });

                    if let Some(InputCapture {
                        button: prev_button, ..
                    }) = self.input_capture
                    {
                        if prev_button == button {
                            self.input_capture = None;
                        }
                    }
                }
            }

            for input in inputs {
                portal_ref.send_input(input)?;
            }
        }

        let cr = self
            .lfb
            .as_surface_mut(CAIRO_FORMAT_ARGB32, self.screen_size)
            .into_cairo();

        cr.rectangle(prev_cursor_sprite.0, prev_cursor_sprite.1, CURSOR_WIDTH, CURSOR_HEIGHT)
            .clip();

        Self::draw_buffers(&cr, self.screen_size, &self.wallpaper, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_sprite.0, self.cursor_sprite.1)
            .paint();

        Ok(())
    }

    pub fn update_buffers<I>(&mut self, buffers: I)
    where
        I: IntoIterator<Item = ScreenBuffer>,
    {
        let mut prev_input_capture = mem::replace(&mut self.input_capture, None);
        self.buffers.clear();

        let buffers = buffers.into_iter();
        if let (_, Some(capacity)) = buffers.size_hint() {
            self.buffers.reserve(capacity);
        }

        for buffer in buffers {
            prev_input_capture = prev_input_capture.and_then(|prev_input_capture| {
                if prev_input_capture.portal_ref == buffer.portal_ref {
                    self.input_capture = Some(InputCapture {
                        pos: buffer.pos,
                        ..prev_input_capture
                    });
                    None
                } else {
                    Some(prev_input_capture)
                }
            });

            self.buffers.push(buffer);
        }

        let cr = self
            .lfb
            .as_surface_mut(CAIRO_FORMAT_ARGB32, self.screen_size)
            .into_cairo();

        Self::draw_buffers(&cr, self.screen_size, &self.wallpaper, &self.buffers);

        cr.reset_clip()
            .set_source_surface(&self.cursor, self.cursor_sprite.0, self.cursor_sprite.1)
            .paint();
    }
}
