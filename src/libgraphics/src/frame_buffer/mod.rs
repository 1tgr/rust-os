use cairo::bindings::{cairo_format_t, CAIRO_FORMAT_RGB24};
use cairo::surface::{CairoSurface, CairoSurfaceMut};
use core::ops::{Deref, DerefMut};

pub trait AsSurface {
    fn as_surface(&self, format: cairo_format_t, size: (u16, u16)) -> CairoSurface;
}

pub trait AsSurfaceMut {
    fn as_surface_mut(&mut self, format: cairo_format_t, size: (u16, u16)) -> CairoSurfaceMut;
}

impl<T> AsSurface for T
where
    T: Deref<Target = [u8]>,
{
    fn as_surface(&self, format: i32, size: (u16, u16)) -> CairoSurface {
        CairoSurface::from_slice(&*self, format, size.0, size.1)
    }
}

impl<T> AsSurfaceMut for T
where
    T: DerefMut<Target = [u8]>,
{
    fn as_surface_mut(&mut self, format: i32, size: (u16, u16)) -> CairoSurfaceMut {
        CairoSurfaceMut::from_slice(&mut *self, format, size.0, size.1)
    }
}

fn byte_len(size: (u16, u16)) -> usize {
    let stride = cairo::stride_for_width(CAIRO_FORMAT_RGB24, size.0);
    stride * size.1 as usize
}

#[cfg(target_os = "rust_os")]
mod rust_os;

#[cfg(not(target_os = "rust_os"))]
mod posix;

#[cfg(target_os = "rust_os")]
pub use rust_os::FrameBuffer;

#[cfg(not(target_os = "rust_os"))]
pub use posix::FrameBuffer;
