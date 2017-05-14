#![feature(link_args)]
#![feature(start)]

extern crate cairo;
extern crate graphics;
extern crate os;
extern crate syscall;

use graphics::{Command,Event};
use os::{File,OSHandle,Result};

fn run() -> Result<()> {
    let mut client2server = File::from_raw(OSHandle::from_raw(2));
    let mut server2client = File::from_raw(OSHandle::from_raw(3));
    println!("[Client] Sending command");
    graphics::send_message(&mut client2server, Command::CreateWindow)?;
    loop {
        println!("[Client] Waiting for event");
        let e : Event = graphics::read_message(&mut server2client)?;
        println!("[Client] Got event: {:?}", e);
    }
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
