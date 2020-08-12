#![no_std]

extern crate alloc;

#[cfg(not(target_os = "rust_os"))]
extern crate cratesio_libc as libc;

#[link(name = "c")]
#[link(name = "cairo")]
#[link(name = "freetype")]
#[link(name = "pixman-1")]
#[link(name = "png16")]
#[link(name = "z")]
#[link(name = "m")]
#[cfg_attr(target_os = "rust_os", link(name = "gcc"))]
extern "C" {}

#[allow(non_camel_case_types)]
pub mod bindings;

mod cairo;
mod font_face;
mod surface;

pub use cairo::Cairo;
pub use font_face::FontFace;
pub use surface::{Surface, SurfaceMut};

use crate::bindings::*;
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;
use core::result;
use core::slice;
use core::str;
use libc::{c_int, c_uint, strlen};

pub struct Error(pub cairo_status_t);

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = unsafe {
            let p = cairo_status_to_string(self.0);
            let len = strlen(p);
            slice::from_raw_parts(p as *const u8, len as usize)
        };

        if let Ok(s) = str::from_utf8(slice) {
            f.write_str(s)
        } else {
            fmt::Debug::fmt(&(self.0 as c_uint), f)
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn stride_for_width(format: cairo_format_t, width: u16) -> usize {
    unsafe { cairo_format_stride_for_width(format, width as c_int) as usize }
}

pub trait CairoDrop {
    unsafe fn drop_cairo(ptr: *mut Self);
    unsafe fn reference_cairo(ptr: *mut Self) -> *mut Self;
}

impl CairoDrop for cairo_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_destroy(ptr)
    }

    unsafe fn reference_cairo(ptr: *mut Self) -> *mut Self {
        cairo_reference(ptr)
    }
}

impl CairoDrop for cairo_font_face_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_font_face_destroy(ptr)
    }

    unsafe fn reference_cairo(ptr: *mut Self) -> *mut Self {
        cairo_font_face_reference(ptr)
    }
}

impl CairoDrop for cairo_pattern_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_pattern_destroy(ptr)
    }

    unsafe fn reference_cairo(ptr: *mut Self) -> *mut Self {
        cairo_pattern_reference(ptr)
    }
}

impl CairoDrop for cairo_surface_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_surface_destroy(ptr)
    }

    unsafe fn reference_cairo(ptr: *mut Self) -> *mut Self {
        cairo_surface_reference(ptr)
    }
}

pub struct CairoObj<T: CairoDrop>(NonNull<T>);

impl<T: CairoDrop> CairoObj<T> {
    pub fn wrap(ptr: *mut T) -> CairoObj<T> {
        assert!(ptr as usize != 0);
        CairoObj(unsafe { NonNull::new_unchecked(ptr) })
    }
}

impl<T: CairoDrop> Clone for CairoObj<T> {
    fn clone(&self) -> Self {
        CairoObj(unsafe { NonNull::new_unchecked(CairoDrop::reference_cairo(self.0.as_ptr())) })
    }
}

impl<T: CairoDrop> Deref for CairoObj<T> {
    type Target = NonNull<T>;

    fn deref(&self) -> &NonNull<T> {
        &self.0
    }
}

impl<T: CairoDrop> Drop for CairoObj<T> {
    fn drop(&mut self) {
        unsafe { CairoDrop::drop_cairo(self.0.as_ptr()) }
    }
}

pub struct CairoFunc<F: FnMut(T1, T2) -> T3, T1, T2, T3>(F, PhantomData<(T1, T2, T3)>);

impl<F: FnMut(T1, T2) -> T3, T1, T2, T3> CairoFunc<F, T1, T2, T3> {
    pub fn new(f: F) -> Self {
        CairoFunc(f, PhantomData)
    }

    extern "C" fn call(closure: *mut libc::c_void, arg1: T1, arg2: T2) -> T3 {
        let closure: &mut CairoFunc<F, T1, T2, T3> = unsafe { &mut *(closure as *mut Self) };
        closure.0(arg1, arg2)
    }

    pub fn func(&self) -> extern "C" fn(*mut libc::c_void, T1, T2) -> T3 {
        Self::call
    }

    pub fn closure(&mut self) -> *mut libc::c_void {
        self as *mut Self as *mut libc::c_void
    }
}
