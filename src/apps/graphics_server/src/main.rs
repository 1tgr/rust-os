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

use core::str;
use graphics::{ServerPipe, ServerApp};
use os::{File, Result, Thread};
use std::io::Read;

fn run() -> Result<()> {
    let app = ServerApp::new()?;

    {
        let input = app.input();

        let run = move || -> Result<()> {
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
        };

        Thread::spawn(move || run().map(|()| 0).unwrap_or_else(|num| -(num as i32)))?;
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
