use bindings::*;
use CairoObj;
use core::ops::Deref;
use libc::{c_int,c_uchar};

pub struct CairoSurface(CairoObj<cairo_surface_t>);

impl CairoSurface {
    pub fn for_data(data: *mut u8, format: cairo_format_t, width: u16, height: u16, stride: usize) -> Self {
        CairoSurface(CairoObj::wrap(unsafe { cairo_image_surface_create_for_data(data as *mut c_uchar, format, width as c_int, height as c_int, stride as c_int) }))
    }
}

impl Deref for CairoSurface {
    type Target = *mut cairo_surface_t;

    fn deref(&self) -> &*mut cairo_surface_t {
        &*self.0
    }
}
