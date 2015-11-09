#![feature(convert)]

extern crate cairo;
extern crate graphics;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use std::io::{Read,Write};
use std::os::{File,OSHandle,Result,SharedMem};

struct Window<'a> {
    shared_mem: SharedMem,
    client2server: &'a mut File,
    format: cairo_format_t,
    width: f64,
    height: f64
}

impl<'a> Window<'a> {
    pub fn open(shared_mem: SharedMem, client2server: &'a mut File, format: cairo_format_t, width: f64, height: f64) -> Result<Self> {
        let mut w = Window {
            shared_mem: shared_mem,
            client2server: client2server,
            format: format,
            width: width,
            height: height
        };

        try!(w.resize());
        Ok(w)
    }

    fn stride(&self) -> usize {
        cairo::stride_for_width(self.format, (self.width + 0.5) as u16)
    }

    fn resize(&mut self) -> Result<()> {
        let new_len = self.stride() * (self.height + 0.5) as usize;
        self.shared_mem.resize(new_len)
    }

    pub fn paint(&mut self, f: &mut FnMut(&Cairo)) -> Result<()> {
        let surface = CairoSurface::from_raw(&*self.shared_mem, self.format, (self.width + 0.5) as u16, (self.height + 0.5) as u16, self.stride());
        f(&Cairo::new(surface));
        try!(self.client2server.write(b"!"));
        Ok(())
    }
}

fn read_byte(server2client: &mut File) -> Result<u8> {
    let mut buf = Vec::new();
    loop {
        match buf.first() {
            Some(&b) => {
                return Ok(b);
            },
            None => {
                let start = buf.len();
                buf.resize(1, 0);

                let len = try!(server2client.read(&mut buf[start..]));
                buf.truncate(len);
            }
        }
    }
}

fn run() -> Result<()> {
    let shared_mem = SharedMem::open(OSHandle::from_raw(2), true);
    let mut server2client = File::from_raw(OSHandle::from_raw(3));
    let mut client2server = File::from_raw(OSHandle::from_raw(4));
    let mut window = try!(Window::open(shared_mem, &mut client2server, CAIRO_FORMAT_ARGB32, 100.0, 100.0));

    let mut paint = {
        let width = window.width;
        let height = window.height;
        let mut i = 0;
        move |cr: &Cairo| {
            cr.rectangle(0.0, 0.0, width, height)
                .set_source_rgb(1.0, 0.0, (i % 100) as f64 * 0.01)
                .fill()
                .set_source_rgb(0.0, 0.0, 0.0)
                .move_to(50.0, 50.0)
                .show_text(format!("i = {}", i).as_str());
            i += 1;
        }
    };

    loop {
        try!(window.paint(&mut paint));
        try!(read_byte(&mut server2client));
    }
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
