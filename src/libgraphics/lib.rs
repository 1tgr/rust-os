#![crate_name = "graphics"]

#![feature(no_std)]
#![feature(libc)]
#![no_std]

extern crate cairo;
extern crate libc;
extern crate syscall;

use cairo::bindings::*;
use cairo::surface::CairoSurface;
use libc::c_void;
use syscall::{Handle,Result};

extern "C" fn free_kernel_surface_data(void: *mut c_void) {
    let _ = syscall::free_pages(void as *mut u8);
}

fn create_kernel_surface(ptr: *mut u8, format: cairo_format_t, width: u16, height: u16) -> CairoSurface {
    let stride = cairo::stride_for_width(format, width);
    let surface = CairoSurface::for_data(ptr, format, width, height, stride);
    static KEY: cairo_user_data_key_t = cairo_user_data_key_t { unused: 0 };
    unsafe { cairo_surface_set_user_data(*surface, &KEY as *const _, ptr as *mut _, Some(free_kernel_surface_data)) };
    surface
}

pub fn create_lfb_surface(format: cairo_format_t, width: u16, height: u16, bpp: u8) -> Result<CairoSurface> {
    let ptr = try!(syscall::init_video_mode(width, height, bpp));
    let surface = create_kernel_surface(ptr, format, width, height);
    Ok(surface)
}

pub fn create_shared_mem_surface(handle: Handle, format: cairo_format_t, width: u16, height: u16) -> Result<CairoSurface> {
    let stride = cairo::stride_for_width(format, width);
    let ptr = try!(syscall::map_shared_mem(handle, stride * height as usize, true));
    let surface = create_kernel_surface(ptr, format, width, height);
    Ok(surface)
}
