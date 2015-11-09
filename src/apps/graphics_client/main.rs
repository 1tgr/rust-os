#![feature(convert)]

extern crate cairo;
extern crate graphics;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use std::io::{Read,Write};
use std::os::{File,OSHandle,Result,SharedMem};

fn run() -> Result<()> {
    const WIDTH: u16 = 100;
    const HEIGHT: u16 = 100;
    const FORMAT: cairo_format_t = CAIRO_FORMAT_ARGB32;

    let mut mem = SharedMem::open(OSHandle::from_raw(2), true);
    let mut server2client = File::from_raw(OSHandle::from_raw(3));
    let mut client2server = File::from_raw(OSHandle::from_raw(4));
    let stride = cairo::stride_for_width(FORMAT, WIDTH);
    try!(mem.resize(stride * HEIGHT as usize));

    let surface = CairoSurface::from_raw(&*mem, FORMAT, WIDTH, HEIGHT, stride);
    let mut i = 0;
    let mut buf = Vec::new();

    loop {
        {
            let cr = Cairo::new(surface.clone());
            cr.rectangle(0.0, 0.0, WIDTH as f64, HEIGHT as f64)
                .set_source_rgb(1.0, 0.0, (i % 100) as f64 * 0.01)
                .fill()
                .set_source_rgb(0.0, 0.0, 0.0)
                .move_to(50.0, 50.0)
                .show_text(format!("i = {}", i).as_str());
        }

        i += 1;
        try!(client2server.write(b"!"));
        buf.resize(1, 0);

        let len = try!(server2client.read(&mut buf[..]));
        buf.truncate(len);
    }
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
