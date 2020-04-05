use cairo;
use cairo::surface::CairoSurface;
use os::{Result, SharedMem};

pub struct FrameBuffer {
    width: f64,
    height: f64,
    shared_mem: SharedMem,
}

impl FrameBuffer {
    pub fn new(width: f64, height: f64, shared_mem: SharedMem) -> Result<Self> {
        let mut buffer = FrameBuffer {
            width,
            height,
            shared_mem,
        };
        buffer.resize_shared_mem()?;
        Ok(buffer)
    }

    pub fn resize(&mut self, width: f64, height: f64) -> Result<()> {
        self.width = width;
        self.height = height;
        self.resize_shared_mem()
    }

    pub fn width_i(&self) -> u16 {
        (self.width + 0.5) as u16
    }

    pub fn height_i(&self) -> u16 {
        (self.height + 0.5) as u16
    }

    pub fn stride(&self) -> usize {
        cairo::stride_for_width(cairo::bindings::CAIRO_FORMAT_ARGB32, self.width_i())
    }

    fn resize_shared_mem(&mut self) -> Result<()> {
        let new_len = self.stride() * self.height_i() as usize;
        self.shared_mem.resize(new_len)
    }

    pub fn as_surface(&mut self) -> CairoSurface {
        let width = self.width_i();
        let height = self.height_i();
        let stride = self.stride();
        CairoSurface::from_slice(
            &mut *self.shared_mem,
            cairo::bindings::CAIRO_FORMAT_ARGB32,
            width,
            height,
            stride,
        )
    }
}
