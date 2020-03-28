use crate::bindings::*;
use crate::CairoObj;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;
use libc::{c_int, c_uchar};

pub struct CairoSurface<'a>(CairoObj<cairo_surface_t>, PhantomData<&'a mut [u8]>);

impl<'a> CairoSurface<'a> {
    pub fn from_slice(data: &'a mut [u8], format: cairo_format_t, width: u16, height: u16, stride: usize) -> Self {
        assert_eq!(data.len(), stride * height as usize);

        let ptr = unsafe {
            cairo_image_surface_create_for_data(
                data.as_ptr() as *mut c_uchar,
                format,
                width as c_int,
                height as c_int,
                stride as c_int,
            )
        };
        CairoSurface(CairoObj::wrap(ptr), PhantomData)
    }
}

impl<'a> Deref for CairoSurface<'a> {
    type Target = NonNull<cairo_surface_t>;

    fn deref(&self) -> &NonNull<cairo_surface_t> {
        &self.0
    }
}
