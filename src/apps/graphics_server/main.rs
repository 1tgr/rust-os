#![feature(collections)]
#![feature(link_args)]
#![feature(start)]

extern crate cairo;
extern crate collections;
extern crate graphics;
extern crate os;
extern crate serde;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use collections::BTreeMap;
use graphics::{Command,FrameBuffer};
use os::{File,OSMem,Process,Result,SharedMem};
use syscall::ErrNum;
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
    let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let mut lfb_mem = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };

    let mut paint_all = |windows: &mut BTreeMap<usize, (f64, f64, FrameBuffer)>| {
        let cr = Cairo::new(CairoSurface::from_raw(&mut lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride));
        for &mut (x, y, ref mut buffer) in windows.values_mut() {
            let window_surface = buffer.create_surface();
            cr.set_source_surface(&window_surface, x, y);
            cr.paint();
        }
    };

    let mut windows = BTreeMap::new();
    let (client_process, mut client2server, mut server2client) = start_client("graphics_client")?;

    loop {
        match graphics::read_message(&mut client2server)? {
            Command::CreateWindow { id, pos, shared_mem_handle } => {
                let shared_mem = SharedMem::from_raw(client_process.open_handle(shared_mem_handle)?, false);
                windows.insert(id, (pos.x, pos.y, FrameBuffer::new(pos.width, pos.height, shared_mem)?));
                paint_all(&mut windows);
            },

            Command::DestroyWindow { ref id } => {
                windows.remove(id);
                paint_all(&mut windows);
            },

            Command::InvalidateWindow { id: ref _id } => {
                paint_all(&mut windows);
            },

            Command::MoveWindow { ref id, ref pos } => {
                {
                    let &mut (ref mut x, ref mut y, ref mut buffer) = windows.get_mut(id).ok_or(ErrNum::FileNotFound)?;
                    *x = pos.x;
                    *y = pos.y;
                    buffer.resize(pos.width, pos.height)?;
                }

                paint_all(&mut windows);
            }
        }
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
