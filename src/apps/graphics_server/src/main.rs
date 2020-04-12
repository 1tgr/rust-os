#![feature(link_args)]
#![feature(start)]

extern crate alloc;
extern crate alloc_system;
extern crate cairo;
extern crate core;
extern crate graphics;
extern crate os;
extern crate rt;
extern crate serde;
extern crate syscall;

use core::mem;
use core::str;
use graphics::{ServerApp, ServerInput, ServerPipe};
use os::{File, Result, Thread};
use std::io::Read;

fn keyboard_thread(input: ServerInput) -> Result<()> {
    let mut stdin = File::open("stdin")?;
    let mut buf = [0; 4];
    loop {
        let len = stdin.read(&mut buf)?;
        if let Ok(s) = str::from_utf8(&buf[..len]) {
            if let Some(c) = s.chars().next() {
                input.send_keypress(c)?;
            }
        }
    }
}

fn mouse_thread(input: ServerInput) -> Result<()> {
    let mut mouse = File::open("ps2_mouse")?;
    let mut buf = [0; 6];
    loop {
        let len = mouse.read(&mut buf)?;
        assert_eq!(len, buf.len());

        #[derive(Debug)]
        struct MouseEvent {
            dx: i16,
            dy: i16,
            dw: i8,
            buttons: u8,
        }

        let event = unsafe { mem::transmute::<[u8; 6], MouseEvent>(buf) };
        let left = (event.buttons & 1) != 0;
        let middle = (event.buttons & 2) != 0;
        let right = (event.buttons & 4) != 0;
        input.update_mouse_state(event.dx, event.dy, event.dw, left, middle, right);
    }
}

fn run() -> Result<()> {
    let app = ServerApp::new()?;

    {
        let input = app.input();
        Thread::spawn(move || keyboard_thread(input).map(|()| 0).unwrap_or_else(|num| -(num as i32)))?;
    }

    {
        let input = app.input();
        Thread::spawn(move || mouse_thread(input).map(|()| 0).unwrap_or_else(|num| -(num as i32)))?;
    }

    ServerPipe::new(app, "graphics_client")?.run()
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
