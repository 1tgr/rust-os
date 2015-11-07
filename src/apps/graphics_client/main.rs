#![feature(libc)]

extern crate cairo;
extern crate libc;
extern crate syscall;

use cairo::*;
use libc::c_int;
use syscall::Result;

fn run() -> Result<()> {
    const WIDTH: c_int = 100;
    const HEIGHT: c_int = 100;
    const FORMAT: cairo_format_t = CAIRO_FORMAT_ARGB32;

    unsafe {
        let stride = cairo_format_stride_for_width(FORMAT, WIDTH);
        let lfb_ptr = try!(syscall::map_shared_mem(2, stride as usize * HEIGHT as usize, true));
        {
            let surface = CairoObj::wrap(cairo_image_surface_create_for_data(lfb_ptr, FORMAT, WIDTH, HEIGHT, stride));
            let cr = CairoObj::wrap(cairo_create(*surface));
            cairo_rectangle(*cr, 0.0, 0.0, WIDTH as f64, HEIGHT as f64);
            cairo_set_source_rgb(*cr, 0.0, 0.0, 1.0);
            cairo_fill(*cr);
        }

        let _ = syscall::free_pages(lfb_ptr);
    };

    Ok(())
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
