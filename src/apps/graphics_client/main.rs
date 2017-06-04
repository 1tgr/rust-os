#![feature(collections)]
#![feature(link_args)]
#![feature(start)]
#![feature(unique)]

extern crate cairo;
extern crate collections;
extern crate graphics;
extern crate os;
extern crate syscall;

use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::CairoObj;
use collections::btree_map::BTreeMap;
use graphics::{Client,Event,Window};
use os::{Mutex,Result};

fn run() -> Result<()> {
    let client = Mutex::new(Client::new())?;
    let mut windows = BTreeMap::new();
    for i in 0 .. 5 {
        let mut window = Window::new(&client, i as f64 * 100.0, i as f64 * 100.0, 150.0, 120.0)?;

        {
            let surface = window.create_surface();
            let cr = Cairo::new(surface);
            unsafe {
                let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 100.0));
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
                cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 150.0, 120.0);
                cairo_set_source(cr.as_ptr(), pat.as_ptr());
                cairo_fill(cr.as_ptr());
            }
        }

        window.invalidate()?;
        windows.insert(window.id(), window);
    }

    let checkpoint_id = {
        let mut client = client.lock().unwrap();
        client.checkpoint()?
    };

    while !windows.is_empty() {
        let e = {
            let mut client = client.lock().unwrap();
            client.wait_for_event()?
        };

        println!("[Client] {:?}", e);
        match e {
            Event::Checkpoint { id } if id == checkpoint_id => {
                println!("System ready");
            },

            Event::KeyPress { window_id, code } if code as u32 == 27 => {
                windows.remove(&window_id);
            },

            _ => { }
        }
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
