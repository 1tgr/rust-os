use bindings::*;
use CairoObj;
use core::clone::Clone;
use core::marker::PhantomData;
use core::ops::Deref;
use libc::{c_int,c_uchar};

pub struct CairoSurface<'a>(CairoObj<cairo_surface_t>, PhantomData<&'a u8>);

impl<'a> CairoSurface<'a> {
    pub fn from_raw<T: Deref<Target=*mut u8>>(data: &T, format: cairo_format_t, width: u16, height: u16, stride: usize) -> Self {
        let ptr: *mut u8 = **data;
        CairoSurface(CairoObj::wrap(unsafe { cairo_image_surface_create_for_data(ptr as *mut c_uchar, format, width as c_int, height as c_int, stride as c_int) }), PhantomData)
    }
}

impl<'a> Clone for CairoSurface<'a> {
    fn clone(&self) -> Self {
        CairoSurface(self.0.clone(), PhantomData)
    }
}

impl<'a> Deref for CairoSurface<'a> {
    type Target = *mut cairo_surface_t;

    fn deref(&self) -> &*mut cairo_surface_t {
        &*self.0
    }
}
