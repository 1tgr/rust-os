#![feature(link_args)]
#![feature(rustc_private)]
#![feature(start)]

extern crate alloc;
extern crate alloc_system;
extern crate rt;

use alloc::rc::Rc;
use cairo::bindings::*;
use cairo::CairoObj;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use ecs::Entity;
use graphics::{App, ClientPortal, Event, Rect};
use os::Result;

fn run() -> Result<()> {
    let mut app = App::new()?;
    let entities = app.entities_mut();
    for i in 0..5 {
        let portal = Entity::new();
        let text = Rc::new(RefCell::new("hello".to_owned()));
        let portal_state = ClientPortal::new(
            entities,
            Rect {
                x: i as f64 * 100.0,
                y: i as f64 * 100.0,
                width: 150.0,
                height: 120.0,
            },
            {
                let text = text.clone();
                move |cr| unsafe {
                    let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 100.0));
                    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
                    cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
                    cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 150.0, 120.0);
                    cairo_set_source(cr.as_ptr(), pat.as_ptr());
                    cairo_fill(cr.as_ptr());

                    let text = text.borrow();
                    let mut v = Vec::<u8>::with_capacity(text.len() + 1);
                    v.extend(text.as_bytes());
                    v.push(0);

                    let text_ptr = (&v[..]).as_ptr() as *const i8;
                    let mut extents = MaybeUninit::uninit();
                    cairo_text_extents(cr.as_ptr(), text_ptr, extents.as_mut_ptr());

                    let extents = extents.assume_init();
                    cairo_set_source_rgb(cr.as_ptr(), 0.0, 0.0, 0.0);
                    cairo_move_to(cr.as_ptr(), 0.0, extents.height);
                    cairo_show_text(cr.as_ptr(), text_ptr);
                }
            },
            {
                let portal = portal.clone();
                let text = text.clone();
                move |e, code| {
                    match code {
                        '\x08' => {
                            text.borrow_mut().pop();
                        }
                        '\u{1b}' => {
                            e.clear_entity(&portal);
                            return;
                        }
                        _ => {
                            text.borrow_mut().push(code);
                        }
                    }

                    e.update_component(&portal, |state: &mut ClientPortal| {
                        state.invalidate();
                        Ok(())
                    })
                    .unwrap();
                }
            },
        )?;

        entities.add_component(&portal, portal_state);
    }

    let checkpoint_id = app.checkpoint()?;

    loop {
        let e = app.wait_for_event()?;
        println!("[Client] {:?}", e);
        if let Event::Checkpoint { id } = e {
            if id == checkpoint_id {
                println!("System ready");
            }
        }

        app.dispatch_event(&e);
    }
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
