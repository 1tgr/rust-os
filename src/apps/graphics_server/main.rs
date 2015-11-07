extern crate cairo;
extern crate graphics;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use std::mem;
use syscall::{Handle,Result};
use syscall::libc_helpers::{stdin,stdout};

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
    const FORMAT: cairo_format_t = CAIRO_FORMAT_ARGB32;

    let shared_surface = try!(graphics::create_shared_mem_surface(2, FORMAT, 100, 100));
    try!(run_client(shared_mem));

    Cairo::new(try!(graphics::create_lfb_surface(FORMAT, 800, 600, 32)))
        .set_source_surface(shared_surface, 100.0, 100.0)
        .paint();

    Ok(())
}

#[no_mangle]
pub fn main() -> i32 {
    run().map(|()| 0).unwrap_or_else(|num| -(num as i32))
}
