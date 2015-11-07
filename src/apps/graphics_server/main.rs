#![feature(libc)]

extern crate cairo;
extern crate libc;
extern crate syscall;

use cairo::*;
use libc::c_int;
use std::mem;
use syscall::libc_helpers::{stdin,stdout};
use syscall::{Handle,Result};

fn run_client(shared_mem: Handle) -> Result<()> {
    let inherit = unsafe { [ stdin, stdout, shared_mem ] };
    let process = try!(syscall::spawn("graphics_client", &inherit));
    match syscall::wait_for_exit(process) {
        Err(num) => { return Err(num); },
        Ok(code) if code < 0 => { return Err(unsafe { mem::transmute(-code as usize) }) },
        Ok(_) => ()
    }

    let _ = syscall::close(process);
    Ok(())
}

fn run() -> Result<()> {
    let shared_mem = try!(syscall::create_shared_mem());
    const WIDTH: c_int = 100;
    const HEIGHT: c_int = 100;
    const FORMAT: cairo_format_t = CAIRO_FORMAT_ARGB32;

    unsafe {
        let shared_stride = cairo_format_stride_for_width(FORMAT, WIDTH);
        let shared_ptr = try!(syscall::map_shared_mem(2, shared_stride as usize * HEIGHT as usize, true));
        {
            let shared_surface = CairoObj::wrap(cairo_image_surface_create_for_data(shared_ptr, FORMAT, WIDTH, HEIGHT, shared_stride));
            try!(run_client(shared_mem));

            let lfb_ptr = try!(syscall::init_video_mode(800, 600, 32));
            {
                let lfb_surface = {
                    let lfb_stride = cairo_format_stride_for_width(CAIRO_FORMAT_ARGB32, 800);
                    CairoObj::wrap(cairo_image_surface_create_for_data(lfb_ptr, FORMAT, WIDTH, HEIGHT, lfb_stride))
                };

                let cr = CairoObj::wrap(cairo_create(*lfb_surface));
                cairo_set_source_surface(*cr, *shared_surface, 0.0, 0.0);
                cairo_paint(*cr);
            }

            let _ = syscall::free_pages(lfb_ptr);
        }

        let _ = syscall::free_pages(shared_ptr);
    }

    Ok(())
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
