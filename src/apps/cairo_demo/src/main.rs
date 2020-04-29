#![feature(link_args)]
#![feature(start)]

extern crate alloc_system;
extern crate cairo;
extern crate libc;
extern crate os;
extern crate rt;
extern crate syscall;

use cairo::bindings::*;
use cairo::surface::CairoSurfaceMut;
use cairo::{CairoFunc, CairoObj};
use os::OSMem;
use std::f64::consts;
use std::mem::MaybeUninit;
use std::ptr;

unsafe fn main() -> isize {
    let lfb_ptr = syscall::init_video_mode(800, 600, 32).unwrap();
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let mut lfb = OSMem::from_raw(lfb_ptr, stride * 600);
    let cr = CairoSurfaceMut::from_slice(&mut lfb, CAIRO_FORMAT_ARGB32, 800, 600).into_cairo();

    let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 256.0));
    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
    cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 256.0, 256.0);
    cairo_set_source(cr.as_ptr(), pat.as_ptr());
    cairo_fill(cr.as_ptr());

    let pat = CairoObj::wrap(cairo_pattern_create_radial(115.2, 102.4, 25.6, 102.4, 102.4, 128.0));
    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
    cairo_set_source(cr.as_ptr(), pat.as_ptr());
    cairo_arc(cr.as_ptr(), 128.0, 128.0, 76.8, 0.0, 2.0 * consts::PI);
    cairo_fill(cr.as_ptr());

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

    let image = CairoObj::wrap(cairo_image_surface_create_from_png_stream(
        Some(reader.func()),
        reader.closure(),
    ));
    cairo_set_source_surface(
        cr.as_ptr(),
        image.as_ptr(),
        (128 - cairo_image_surface_get_width(image.as_ptr()) / 2) as f64,
        (128 - cairo_image_surface_get_height(image.as_ptr()) / 2) as f64,
    );

    cairo_paint(cr.as_ptr());

    let message = b"Hello, world\0".as_ptr() as *const i8;
    let mut extents = MaybeUninit::uninit();
    cairo_text_extents(cr.as_ptr(), message, extents.as_mut_ptr());

    let extents = extents.assume_init();
    cairo_set_source_rgb(cr.as_ptr(), 0.0, 0.0, 0.0);
    cairo_move_to(
        cr.as_ptr(),
        128.0 - extents.x_advance / 2.0,
        128.0 + ((cairo_image_surface_get_height(image.as_ptr()) / 2) as f64) + extents.height,
    );

    cairo_show_text(cr.as_ptr(), message);
    0
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    unsafe { main() }
}
