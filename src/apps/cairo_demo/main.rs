#![feature(libc)]

extern crate cairo;
extern crate libc;
extern crate syscall;

use cairo::{CairoFunc,CairoObj};
use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use std::f64::consts;
use std::mem;
use std::os::OSMem;
use std::ptr;

#[no_mangle]
pub unsafe fn main() -> i32 {
    let lfb = OSMem::from_raw(syscall::init_video_mode(800, 600, 32).unwrap());
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let surface = CairoSurface::from_raw(&lfb, CAIRO_FORMAT_ARGB32, 800, 600, stride);

    let cr = Cairo::new(surface);
    let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 256.0));
    cairo_pattern_add_color_stop_rgba(*pat, 1.0, 0.0, 0.0, 0.0, 1.0);
    cairo_pattern_add_color_stop_rgba(*pat, 0.0, 1.0, 1.0, 1.0, 1.0);
    cairo_rectangle(*cr, 0.0, 0.0, 256.0, 256.0);
    cairo_set_source(*cr, *pat);
    cairo_fill(*cr);

    let pat = CairoObj::wrap(cairo_pattern_create_radial(115.2, 102.4, 25.6, 102.4, 102.4, 128.0));
    cairo_pattern_add_color_stop_rgba(*pat, 0.0, 1.0, 1.0, 1.0, 1.0);
    cairo_pattern_add_color_stop_rgba(*pat, 1.0, 0.0, 0.0, 0.0, 1.0);
    cairo_set_source(*cr, *pat);
    cairo_arc(*cr, 128.0, 128.0, 76.8, 0.0, 2.0 * consts::PI);
    cairo_fill(*cr);

    let slice = include_bytes!("gustavo.png");

    let mut reader = {
        let mut offset = 0;
        CairoFunc::new(move |data: *mut u8, length: libc::c_uint| -> cairo_status_t {
            let length = length as usize;
            if offset + length > slice.len() {
                return CAIRO_STATUS_READ_ERROR;
            }

            ptr::copy_nonoverlapping(slice.as_ptr().offset(offset as isize), data, length);
            offset += length;
            CAIRO_STATUS_SUCCESS
        })
    };

    let image = CairoObj::wrap(cairo_image_surface_create_from_png_stream(Some(reader.func()), reader.closure()));
    cairo_set_source_surface(*cr, *image, (128 - cairo_image_surface_get_width(*image) / 2) as f64, (128 - cairo_image_surface_get_height(*image) / 2) as f64);
    cairo_paint(*cr);

    let message = b"Hello, world\0".as_ptr() as *const i8;
    let mut extents = mem::uninitialized();
    cairo_text_extents(*cr, message, &mut extents);
    cairo_set_source_rgb(*cr, 0.0, 0.0, 0.0);
    cairo_move_to(*cr, 128.0 - extents.x_advance / 2.0, 128.0 + ((cairo_image_surface_get_height(*image) / 2) as f64) + extents.height);
    cairo_show_text(*cr, message);
    0x1234
}
