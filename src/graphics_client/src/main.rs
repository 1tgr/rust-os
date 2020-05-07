#[cfg(target_os = "rust_os")]
extern crate alloc_system;

#[cfg(target_os = "rust_os")]
extern crate rt;

use core::fmt::Write;
use graphics::components::{NeedsPaint, OnClick, OnInput, Parent, Position, Text};
use graphics::widgets::{Button, ClientPortal, Label, TextBox};
use graphics::{App, Event, Result};

fn main() -> Result<()> {
    let mut app = App::new();
    let world = app.world_mut();
    for i in 0..5 {
        let label = world.spawn((
            Label,
            Position::new(0.0, 0.0, 300.0, 10.0),
            Text::new(format!("[{}] hello", i)),
        ));

        let portal = world.spawn((
            ClientPortal,
            Position::new(i as f64 * 100.0, i as f64 * 100.0, 300.0, 120.0),
            OnInput::new(move |world, _entity, input| {
                {
                    let Text(text) = &mut *world.get_mut::<Text>(label).unwrap();
                    text.clear();
                    write!(text, "[{}] {:?}", i, input).unwrap();
                }

                world.insert_one(label, NeedsPaint).unwrap();
                Ok(())
            }),
        ));

        world.insert_one(label, Parent(portal)).unwrap();

        world.spawn((
            Button,
            Text::new("Close"),
            Parent(portal),
            Position::new(10.0, 10.0, 50.0, 20.0),
            OnClick::new(move |world, _entity| {
                world.despawn(portal).unwrap();
                Ok(())
            }),
        ));

        world.spawn((
            Label,
            Text::new("Label:"),
            Parent(portal),
            Position::new(10.0, 40.0, 50.0, 20.0),
        ));

        world.spawn((TextBox, Parent(portal), Position::new(60.0, 40.0, 230.0, 20.0)));
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
