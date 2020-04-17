use cairo;
use cairo::surface::CairoSurface;
use core::slice;
use os::{OSMem, Result, SharedMem};

pub enum Buffer {
    OSMem(OSMem),
    SharedMem(SharedMem),
    Ptr(*mut u8),
}

pub struct FrameBuffer {
    pub width: f64,
    pub height: f64,
    pub buffer: Buffer,
}

fn byte_len(width: f64, height: f64) -> usize {
    let width = (width + 0.5) as u16;
    let height = (height + 0.5) as u16;
    let stride = cairo::stride_for_width(cairo::bindings::CAIRO_FORMAT_ARGB32, width);
    stride * height as usize
}

impl FrameBuffer {
    pub fn from_os_mem(width: f64, height: f64, os_mem: OSMem) -> Self {
        Self {
            width,
            height,
            buffer: Buffer::OSMem(os_mem),
        }
    }

    pub fn from_shared_mem(width: f64, height: f64, mut shared_mem: SharedMem) -> Result<Self> {
        let len = byte_len(width, height);
        shared_mem.resize(len)?;

        Ok(Self {
            width,
            height,
            buffer: Buffer::SharedMem(shared_mem),
        })
    }

    pub unsafe fn from_ptr(width: f64, height: f64, ptr: *mut u8) -> Self {
        Self {
            width,
            height,
            buffer: Buffer::Ptr(ptr),
        }
    }

    pub fn resize(&mut self, width: f64, height: f64) -> Result<()> {
        let len = byte_len(width, height);
        match &mut self.buffer {
            Buffer::OSMem(_) => panic!("can't resize OSMem"),
            Buffer::SharedMem(shared_mem) => shared_mem.resize(len)?,
            Buffer::Ptr(_) => panic!("can't resize Ptr"),
        }

        self.width = width;
        self.height = height;
        Ok(())
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

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let height = self.height_i();
        let stride = self.stride();

        match self.buffer {
            Buffer::OSMem(ref mut os_mem) => &mut *os_mem,
            Buffer::SharedMem(ref mut shared_mem) => &mut *shared_mem,
            Buffer::Ptr(ptr) => {
                let len = stride * height as usize;
                unsafe { slice::from_raw_parts_mut(ptr, len) }
            }
        }
    }

    pub fn as_surface(&mut self) -> CairoSurface {
        let width = self.width_i();
        let height = self.height_i();
        let stride = self.stride();
        let data = self.as_mut_slice();
        CairoSurface::from_slice(data, cairo::bindings::CAIRO_FORMAT_RGB24, width, height, stride)
    }
}
