#![feature(link_args)]
#![feature(rustc_private)]
#![feature(start)]

extern crate alloc;
extern crate alloc_system;
extern crate cairo;
extern crate core;
extern crate graphics;
extern crate os;
extern crate rt;
extern crate syscall;

use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::CairoObj;
use core::mem::MaybeUninit;
use graphics::{Client, Event, Window};
use os::Result;

struct DemoWindow<'a> {
    window: Window<'a>,
    text: String,
}

impl<'a> DemoWindow<'a> {
    fn new(window: Window<'a>) -> Result<Self> {
        let mut demo_window = DemoWindow {
            window,
            text: "hello".into(),
        };
        demo_window.invalidate()?;
        Ok(demo_window)
    }

    fn invalidate(&mut self) -> Result<()> {
        {
            let surface = self.window.create_surface();
            let cr = Cairo::new(surface);
            unsafe {
                let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 100.0));
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
                cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 150.0, 120.0);
                cairo_set_source(cr.as_ptr(), pat.as_ptr());
                cairo_fill(cr.as_ptr());

                let mut text = Vec::<u8>::with_capacity(self.text.len() + 1);
                text.extend(self.text.as_bytes());
                text.push(0);

                let text_ptr = (&text[..]).as_ptr() as *const i8;
                let mut extents = MaybeUninit::uninit();
                cairo_text_extents(cr.as_ptr(), text_ptr, extents.as_mut_ptr());

                let extents = extents.assume_init();
                cairo_set_source_rgb(cr.as_ptr(), 0.0, 0.0, 0.0);
                cairo_move_to(cr.as_ptr(), 0.0, extents.height);
                cairo_show_text(cr.as_ptr(), text_ptr);
            }
        }

        self.window.invalidate()?;
        Ok(())
    }

    fn handle_keypress(&mut self, code: char) -> Result<()> {
        match code {
            '\x08' => {
                self.text.pop();
            }
            _ => {
                self.text.push(code);
            }
        }

        self.invalidate()?;
        Ok(())
    }
}

fn run() -> Result<()> {
    let client = Client::new();
    let mut windows = BTreeMap::new();
    for i in 0..5 {
        let window = Window::new(&client, i as f64 * 100.0, i as f64 * 100.0, 150.0, 120.0)?;
        windows.insert(window.id(), DemoWindow::new(window)?);
    }

    let checkpoint_id = client.checkpoint()?;

    while !windows.is_empty() {
        let e = client.wait_for_event()?;
        println!("[Client] {:?}", e);
        match e {
            Event::Checkpoint { id } if id == checkpoint_id => {
                println!("System ready");
            }

            Event::KeyPress { window_id, code } if code == '\u{1b}' => {
                windows.remove(&window_id);
            }

            Event::KeyPress { window_id, code } => {
                if let Some(demo_window) = windows.get_mut(&window_id) {
                    demo_window.handle_keypress(code)?;
                }
            }

            _ => {}
        }
    }

    Ok(())
}

#[cfg(target_arch = "x86_64")]
#[allow(unused_attributes)]
#[link_args = "-T ../../libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
