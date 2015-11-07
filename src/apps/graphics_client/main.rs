extern crate cairo;
extern crate graphics;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use syscall::Result;

fn run() -> Result<()> {
    const WIDTH: u16 = 100;
    const HEIGHT: u16 = 100;
    const FORMAT: cairo_format_t = CAIRO_FORMAT_ARGB32;

    Cairo::new(try!(graphics::create_shared_mem_surface(2, FORMAT, WIDTH, HEIGHT)))
        .rectangle(0.0, 0.0, WIDTH as f64, HEIGHT as f64)
        .set_source_rgb(0.0, 0.0, 1.0)
        .fill();

    Ok(())
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
