extern crate cairo;
extern crate graphics;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use std::io::Read;
use std::os::{File,OSHandle,OSMem,Result,SharedMem};
use syscall::libc_helpers;

struct Window {
    shared_mem: SharedMem,
    x: f64,
    y: f64,
    format: cairo_format_t,
    width: f64,
    height: f64
}

impl Window {
    pub fn new(format: cairo_format_t, x: f64, y: f64, width: f64, height: f64) -> Result<Self> {
        let mut w = Window {
            shared_mem: try!(SharedMem::create(false)),
            x: x,
            y: y,
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

    pub fn draw_to(&self, cr: &Cairo) {
        let surface = CairoSurface::from_raw(&*self.shared_mem, self.format, (self.width + 0.5) as u16, (self.height + 0.5) as u16, self.stride());
        cr.set_source_surface(surface, self.x, self.y).paint();
    }
}

fn start_client(window: &Window, client2server: &File) -> Result<OSHandle> {
    let (stdin, stdout) = unsafe { (libc_helpers::stdin, libc_helpers::stdout) };
    let inherit = [
        stdin,
        stdout,
        **window.shared_mem.handle(),
        stdin,
        **client2server.handle()
    ];

    Ok(OSHandle::from_raw(try!(syscall::spawn("graphics_client", &inherit))))
}

fn run() -> Result<()> {
    let window = try!(Window::new(CAIRO_FORMAT_ARGB32, 50.0, 50.0, 100.0, 100.0));
    let mut client2server = try!(File::create_pipe());
    let _process = try!(start_client(&window, &client2server));

    let lfb_mem = OSMem::from_raw(try!(syscall::init_video_mode(800, 600, 32)));
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let cr = Cairo::new(CairoSurface::from_raw(&lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride));
    let mut buf = Vec::new();
    loop {
        buf.resize(1, 0);

        let len = try!(client2server.read(&mut buf[..]));
        buf.truncate(len);
        window.draw_to(&cr);
    }
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
