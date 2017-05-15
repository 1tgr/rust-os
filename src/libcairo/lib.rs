#![crate_name = "cairo"]

#![feature(collections)]
#![feature(unique)]
#![no_std]

extern crate collections;
extern crate libc;

#[link(name = "c")]
#[link(name = "cairo")]
#[link(name = "pixman-1")]
#[link(name = "png16")]
#[link(name = "z")]
#[link(name = "gcc")]
#[link(name = "m")]
extern {
}

pub mod cairo;
pub mod bindings;
pub mod surface;

use bindings::*;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::Unique;
use libc::c_int;

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

pub struct CairoObj<T: CairoDrop>(Unique<T>);

impl<T: CairoDrop> CairoObj<T> {
    pub fn wrap(ptr: *mut T) -> CairoObj<T> {
        assert!(ptr as usize != 0);
        CairoObj(unsafe { Unique::new(ptr) })
    }
}

impl<T: CairoDrop> Clone for CairoObj<T> {
    fn clone(&self) -> Self {
        CairoObj(unsafe { Unique::new(CairoDrop::reference_cairo(self.0.as_ptr())) })
    }
}

impl<T: CairoDrop> Deref for CairoObj<T> {
    type Target = Unique<T>;

    fn deref(&self) -> &Unique<T> {
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

    pub fn func(&self) -> (extern "C" fn(*mut libc::c_void, T1, T2) -> T3) {
        Self::call
    }

    pub fn closure(&mut self) -> *mut libc::c_void {
        self as *mut Self as *mut libc::c_void
    }
}
