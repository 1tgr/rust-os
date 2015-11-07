#![crate_name = "cairo"]

#![feature(libc)]
#![feature(no_std)]
#![feature(unique)]
#![no_std]

extern crate libc;

mod cairo;

pub use cairo::*;

use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::Unique;

pub trait CairoDrop {
    unsafe fn drop_cairo(ptr: *mut Self);
}

impl CairoDrop for cairo_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_destroy(ptr)
    }
}

impl CairoDrop for cairo_pattern_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_pattern_destroy(ptr)
    }
}

impl CairoDrop for cairo_surface_t {
    unsafe fn drop_cairo(ptr: *mut Self) {
        cairo_surface_destroy(ptr)
    }
}

pub struct CairoObj<T: CairoDrop>(Unique<T>);

impl<T: CairoDrop> CairoObj<T> {
    pub fn wrap(ptr: *mut T) -> CairoObj<T> {
        assert!(ptr as usize != 0);
        CairoObj(unsafe { Unique::new(ptr) })
    }
}

impl<T: CairoDrop> Deref for CairoObj<T> {
    type Target = *mut T;

    fn deref(&self) -> &*mut T {
        &*self.0
    }
}

impl<T: CairoDrop> Drop for CairoObj<T> {
    fn drop(&mut self) {
        unsafe { CairoDrop::drop_cairo(*self.0) }
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
