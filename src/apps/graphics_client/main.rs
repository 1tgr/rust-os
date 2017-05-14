#![feature(link_args)]
#![feature(start)]
#![feature(unique)]

extern crate cairo;
extern crate graphics;
extern crate os;
extern crate syscall;

use cairo::CairoObj;
use cairo::bindings::*;
use cairo::cairo::Cairo;
use graphics::{Client,Window};
use os::Result;
use std::cell::RefCell;

fn run() -> Result<()> {
    let client = RefCell::new(Client::new());
    let mut windows = Vec::new();
    for i in 0 .. 5 {
        let mut window = Window::new(&client, i as f64 * 100.0, i as f64 * 100.0, 150.0, 120.0)?;

        {
            let surface = window.create_surface();
            let cr = Cairo::new(surface);
            unsafe {
                let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 100.0));
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
                cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 150.0, 120.0);
                cairo_set_source(cr.as_ptr(), pat.as_ptr());
                cairo_fill(cr.as_ptr());
            }
        }

        window.invalidate()?;
        windows.push(window);
    }

    loop {
        client.borrow_mut().wait_for_event()?;
    }
}

#[cfg(target_arch="x86_64")]
#[link_args = "-T ../../libsyscall/arch/amd64/link.ld"]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
