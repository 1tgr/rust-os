use bindings::*;
use CairoObj;
use core::marker::PhantomData;
use core::ops::{Deref,DerefMut};
use core::ptr::Unique;
use libc::{c_int,c_uchar};

pub struct CairoSurface<'a>(CairoObj<cairo_surface_t>, PhantomData<&'a u8>);

impl<'a> CairoSurface<'a> {
    pub fn from_raw<T: DerefMut<Target=[u8]> + 'a>(data: &mut T, format: cairo_format_t, width: u16, height: u16, stride: usize) -> Self {
        let data_slice = data.deref_mut();
        let data_ptr = data_slice.as_ptr();
        CairoSurface(CairoObj::wrap(unsafe { cairo_image_surface_create_for_data(data_ptr as *mut c_uchar, format, width as c_int, height as c_int, stride as c_int) }), PhantomData)
    }
}

impl<'a> Deref for CairoSurface<'a> {
    type Target = Unique<cairo_surface_t>;

    fn deref(&self) -> &Unique<cairo_surface_t> {
        &self.0
    }
}
