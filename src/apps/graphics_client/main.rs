#![feature(link_args)]
#![feature(start)]

extern crate cairo;
extern crate graphics;
extern crate os;
extern crate syscall;

use graphics::{Client,Window};
use os::Result;
use std::cell::RefCell;

fn run() -> Result<()> {
    let client = RefCell::new(Client::new());
    println!("[Client] Sending command");
    let window = Window::new(&client, 0.0, 0.0, 100.0, 100.0)?;
    loop {
        println!("[Client] Waiting for event");
        let e = client.borrow_mut().wait_for_event()?;
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
