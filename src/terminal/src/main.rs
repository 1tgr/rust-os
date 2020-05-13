#[cfg(target_os = "rust_os")]
extern crate alloc_system;

#[cfg(target_os = "rust_os")]
extern crate rt;

use crate::state::TerminalState;
use cairo::bindings::{CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL};
use core::cell::RefCell;
use core::str;
use graphics::components::{NeedsPaint, OnInput, OnPaint, Position, Text};
use graphics::widgets::ClientPortal;
use graphics::{App, Event, EventInput, Result};
use os::{File, Process, Thread};
use std::io::{Read, Write};

mod state;

fn main() -> Result<()> {
    let mut app = App::new();
    let stdin = File::create_pipe();
    let mut stdout = File::create_pipe();
    Process::spawn("input", &[stdin.handle().get(), stdout.handle().get()])?;

    let entity = app.world_mut().spawn((
        ClientPortal,
        Text::new("Terminal"),
        Position::new(50.0, 50.0, 700.0, 500.0),
        TerminalState::new(80),
        OnPaint::new(|world, entity, cr| {
            let mut query = world.query_one::<&TerminalState>(entity).unwrap();
            let state = query.get().unwrap();
            cr.select_font_face("monospace", CAIRO_FONT_SLANT_NORMAL, CAIRO_FONT_WEIGHT_NORMAL);

            let extents = cr.font_extents();
            for (index, line) in state.lines().enumerate() {
                cr.move_to(0.0, (index + 1) as f64 * extents.height).show_text(line);
            }
        }),
        OnInput::new({
            let stdin = RefCell::new(stdin);
            move |world, entity, input| {
                if let EventInput::KeyPress { code } = input {
                    let mut s = String::new();
                    s.push(code);
                    stdin.borrow_mut().write_all(s.as_bytes())?;

                    world
                        .query_one::<&mut TerminalState>(entity)
                        .unwrap()
                        .get()
                        .unwrap()
                        .write(&s);

                    world.insert_one(entity, NeedsPaint).unwrap();
                }

                Ok(())
            }
        }),
    ));

    Thread::spawn({
        let mut sync = app.sync();
        move || {
            let mut buf = [0; 4096];
            loop {
                let len = stdout.read(&mut buf).unwrap();
                if let Ok(s) = str::from_utf8(&buf[..len]) {
                    let s = s.to_owned();
                    sync.call(move |world| {
                        world
                            .query_one::<&mut TerminalState>(entity)
                            .unwrap()
                            .get()
                            .unwrap()
                            .write(&s);

                        world.insert_one(entity, NeedsPaint).unwrap();
                        Ok(())
                    });
                }
            }
        }
    });

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
