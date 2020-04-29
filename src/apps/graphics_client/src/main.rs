#![cfg_attr(target_os = "rust_os", feature(link_args))]
#![cfg_attr(target_os = "rust_os", feature(start))]

#[cfg(target_os = "rust_os")]
extern crate alloc_system;

#[cfg(target_os = "rust_os")]
extern crate rt;

use cairo::bindings::*;
use cairo::CairoObj;
use core::fmt::Write;
use core::mem::MaybeUninit;
use graphics::components::{NeedsPaint, OnInput, OnPaint, Parent, Position};
use graphics::{App, ClientPortal, Event, EventInput, MouseButton, MouseInput, Rect, Result};

struct Text(String);
struct ButtonPressed;

fn main() -> Result<()> {
    let mut app = App::new();
    let world = app.world_mut();
    for i in 0..5 {
        let portal = world.spawn((
            ClientPortal,
            Position(Rect {
                x: i as f64 * 100.0,
                y: i as f64 * 100.0,
                width: 300.0,
                height: 120.0,
            }),
            Text("hello".to_owned()),
            OnPaint::new(move |world, entity, cr| unsafe {
                let pat = CairoObj::wrap(cairo_pattern_create_linear(0.0, 0.0, 0.0, 100.0));
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 1.0, 0.0, 0.0, 0.0, 1.0);
                cairo_pattern_add_color_stop_rgba(pat.as_ptr(), 0.0, 1.0, 1.0, 1.0, 1.0);
                cairo_rectangle(cr.as_ptr(), 0.0, 0.0, 300.0, 120.0);
                cairo_set_source(cr.as_ptr(), pat.as_ptr());
                cairo_fill(cr.as_ptr());

                let Text(text) = &*world.get::<Text>(entity).unwrap();
                let mut s = String::new();
                write!(&mut s, "[{}] ", i).unwrap();
                s.push_str(text);
                s.push('\0');

                let text_ptr = s.as_ptr() as *const i8;
                let mut extents = MaybeUninit::uninit();
                cairo_text_extents(cr.as_ptr(), text_ptr, extents.as_mut_ptr());

                let extents = extents.assume_init();
                cairo_set_source_rgb(cr.as_ptr(), 0.0, 0.0, 0.0);
                cairo_move_to(cr.as_ptr(), 0.0, extents.height);
                cairo_show_text(cr.as_ptr(), text_ptr);
            }),
            OnInput::new(|world, entity, input| {
                match *input {
                    EventInput::KeyPress { code } => {
                        match code {
                            '\x08' => {
                                let Text(text) = &mut *world.get_mut::<Text>(entity).unwrap();
                                text.pop();
                            }
                            '\u{1b}' => {
                                world.despawn(entity).unwrap();
                                return Ok(());
                            }
                            _ => {
                                let Text(text) = &mut *world.get_mut::<Text>(entity).unwrap();
                                text.push(code);
                            }
                        }

                        world.insert_one(entity, NeedsPaint).unwrap();
                    }

                    _ => {
                        {
                            let Text(text) = &mut *world.get_mut::<Text>(entity).unwrap();
                            text.clear();
                            write!(text, "{:?}", input).unwrap();
                        }

                        world.insert_one(entity, NeedsPaint).unwrap();
                    }
                }

                Ok(())
            }),
        ));

        world.spawn((
            Parent(portal),
            Position(Rect {
                x: 10.0,
                y: 10.0,
                width: 50.0,
                height: 20.0,
            }),
            OnPaint::new(|world, entity, cr| {
                if world.get::<ButtonPressed>(entity).is_ok() {
                    cr.set_source_rgb(0.8, 0.0, 0.0);
                } else {
                    cr.set_source_rgb(1.0, 0.0, 0.0);
                }

                cr.paint();
            }),
            OnInput::new(|world, entity, input| {
                match *input {
                    EventInput::Mouse {
                        input:
                            MouseInput::ButtonDown {
                                button: MouseButton::Left,
                            },
                        ..
                    } => {
                        world.insert(entity, (ButtonPressed, NeedsPaint)).unwrap();
                    }

                    EventInput::Mouse {
                        input:
                            MouseInput::ButtonUp {
                                button: MouseButton::Left,
                            },
                        ..
                    } => {
                        world.remove_one::<ButtonPressed>(entity).unwrap();
                        world.insert_one(entity, NeedsPaint).unwrap();
                    }

                    _ => (),
                }

                Ok(())
            }),
        ));
    }

    let checkpoint_id = app.checkpoint()?;

    loop {
        let e = app.wait_for_event()?;
        if let Event::Checkpoint { id } = e {
            if id == checkpoint_id {
                println!("System ready");
            }
        }

        app.dispatch_event(e)?;
    }
}

#[cfg(all(target_os = "rust_os", target_arch = "x86_64"))]
#[allow(unused_attributes)]
#[link_args = "-T libsyscall/arch/amd64/link.ld"]
extern "C" {}

#[cfg(target_os = "rust_os")]
#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    main().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
