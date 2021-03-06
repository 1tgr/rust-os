use crate::bindings::*;
use crate::cairo::Cairo;
use crate::{CairoFunc, CairoObj, Error, Result};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::ops::Deref;
use core::ptr::{self, NonNull};
use libc::{c_int, c_uchar};

pub struct Surface<'a>(CairoObj<cairo_surface_t>, Cow<'a, [u8]>);
pub struct SurfaceMut<'a>(CairoObj<cairo_surface_t>, Cow<'a, [u8]>);

impl<'a> Surface<'a> {
    pub fn from_slice(data: &'a [u8], format: cairo_format_t, width: u16, height: u16) -> Self {
        let stride = crate::stride_for_width(format, width);
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

        Self(CairoObj::wrap(ptr), Cow::Borrowed(data))
    }
}

impl<'a> SurfaceMut<'a> {
    pub fn from_slice(data: &'a mut [u8], format: cairo_format_t, width: u16, height: u16) -> Self {
        let stride = crate::stride_for_width(format, width);
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

        Self(CairoObj::wrap(ptr), Cow::Borrowed(data))
    }

    pub fn into_cairo(self) -> Cairo<'a> {
        Cairo::new(self)
    }
}

impl Surface<'static> {
    pub fn from_png_slice(slice: &[u8]) -> Result<Self> {
        let SurfaceMut(ptr, data) = SurfaceMut::from_png_slice(slice)?;
        Ok(Self(ptr, data))
    }

    pub fn from_vec(data: Vec<u8>, format: cairo_format_t, width: u16, height: u16) -> Self {
        let SurfaceMut(ptr, data) = SurfaceMut::from_vec(data, format, width, height);
        Self(ptr, data)
    }
}

impl SurfaceMut<'static> {
    pub fn from_png_slice(slice: &[u8]) -> Result<Self> {
        let mut reader = {
            let mut offset = 0;
            CairoFunc::new(move |data: *mut u8, length: libc::c_uint| -> cairo_status_t {
                let length = length as usize;
                if offset + length > slice.len() {
                    return CAIRO_STATUS_READ_ERROR;
                }

                unsafe {
                    ptr::copy_nonoverlapping(slice.as_ptr().offset(offset as isize), data, length);
                }
                offset += length;
                CAIRO_STATUS_SUCCESS
            })
        };

        let ptr = unsafe {
            let ptr = cairo_image_surface_create_from_png_stream(Some(reader.func()), reader.closure());

            let status = cairo_surface_status(ptr);
            if status != CAIRO_STATUS_SUCCESS {
                return Err(Error(status));
            }

            ptr
        };

        Ok(Self(CairoObj::wrap(ptr), Cow::Borrowed(&[])))
    }

    pub fn from_vec(data: Vec<u8>, format: cairo_format_t, width: u16, height: u16) -> Self {
        let stride = crate::stride_for_width(format, width);
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

        Self(CairoObj::wrap(ptr), Cow::Owned(data))
    }
}

impl<'a> Deref for Surface<'a> {
    type Target = NonNull<cairo_surface_t>;

    fn deref(&self) -> &NonNull<cairo_surface_t> {
        &self.0
    }
}

impl<'a> Deref for SurfaceMut<'a> {
    type Target = NonNull<cairo_surface_t>;

    fn deref(&self) -> &NonNull<cairo_surface_t> {
        &self.0
    }
}
