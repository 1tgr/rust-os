use crate::components::{Focus, NeedsPaint, OnInput, OnPaint, Parent, Position, Text};
use crate::widgets::WidgetSystem;
use cairo::cairo::Cairo;
use graphics_base::system::System;
use graphics_base::types::{EventInput, MouseButton, MouseInput};
use graphics_base::Result;
use hecs::{Component, Entity, RefMut, World};

/* fn find_parent<Q>(world: &World, entity: Entity) -> Option<Ref<Q>> where Q: Component {
    world.get::<Q>(entity).ok().or_else(||{
        let Parent(parent) = *world.get::<Parent>(entity).ok()?;
        find_parent(world, parent)
    })
} */

fn find_parent_mut<Q>(world: &World, entity: Entity) -> Option<RefMut<Q>>
where
    Q: Component,
{
    world.get_mut::<Q>(entity).ok().or_else(|| {
        let Parent(parent) = *world.get::<Parent>(entity).ok()?;
        find_parent_mut(world, parent)
    })
}

pub struct TextBox;

pub struct TextBoxSystem {
    on_paint: OnPaint,
    on_input: OnInput,
}

impl TextBoxSystem {
    pub fn new() -> Self {
        Self {
            on_paint: OnPaint::new(Self::on_paint),
            on_input: OnInput::new(Self::on_input),
        }
    }

    fn on_paint(world: &World, entity: Entity, cr: &Cairo) {
        let mut query = world.query_one::<(&Position, Option<&Text>)>(entity).unwrap();
        cr.set_source_rgb(1.0, 1.0, 1.0).paint();

        if let Some((&Position(pos), text)) = query.get() {
            cr.set_source_rgb(0.0, 0.0, 0.0)
                .rectangle(0.0, 0.0, pos.width, pos.height)
                .stroke();

            if let Some(Text(ref text)) = text {
                let font_extents = cr.font_extents();
                cr.move_to(
                    (pos.height - font_extents.height) / 2.0,
                    (pos.height + font_extents.height) / 2.0,
                )
                .show_text(text);
            }
        }
    }

    fn on_input(world: &mut World, entity: Entity, input: EventInput) -> Result<()> {
        match input {
            EventInput::Mouse {
                input: MouseInput::ButtonDown {
                    button: MouseButton::Left,
                },
                ..
            } => {
                let Focus(ref mut focus) = *find_parent_mut(world, entity).unwrap();
                *focus = Some(entity);
            }

            EventInput::KeyPress { code } => {
                {
                    let mut text = loop {
                        if let Ok(text) = world.get_mut(entity) {
                            break text;
                        }

                        world.insert_one(entity, Text::new("")).unwrap();
                    };

                    let Text(ref mut text) = &mut *text;
                    if code == '\x08' {
                        text.pop();
                    } else {
                        text.push(code);
                    }
                }

                world.insert_one(entity, NeedsPaint).unwrap();
            }

            _ => (),
        }

        Ok(())
    }
}

impl WidgetSystem for TextBoxSystem {
    type Widget = TextBox;
    type Components = (OnPaint, OnInput);

    fn components(&self) -> Self::Components {
        (self.on_paint.clone(), self.on_input.clone())
    }
}

impl System for TextBoxSystem {
    fn run(&mut self, world: &mut World) -> Result<()> {
        WidgetSystem::run(self, world)
    }
}
