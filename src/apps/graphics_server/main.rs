#![feature(link_args)]
#![feature(start)]

extern crate cairo;
extern crate graphics;
extern crate os;
extern crate serde;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use graphics::Command;
use os::{File,OSMem,Process,Result};
use syscall::libc_helpers;

fn start_client(filename: &str) -> Result<(Process, File, File)> {
    let (stdin, stdout) = unsafe { (libc_helpers::stdin, libc_helpers::stdout) };
    let client2server = File::create_pipe()?;
    let server2client = File::create_pipe()?;
    let inherit = [
        stdin,
        stdout,
        client2server.handle().get(),
        server2client.handle().get(),
    ];

    let process = Process::spawn(filename, &inherit)?;
    Ok((process, client2server, server2client))
}

fn run() -> Result<()> {
    /* let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let mut lfb_mem = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };
    let cr = Cairo::new(CairoSurface::from_raw(&mut lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride)); */
    let (client_process, mut client2server, mut server2client) = start_client("graphics_client")?;
    loop {
        println!("[Server] Waiting for command");
        let c : Command = graphics::read_message(&mut client2server)?;
        println!("[Server] Got command: {:?}", c);
    }
    client_process.wait_for_exit()?;
    Ok(())
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
