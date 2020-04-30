use crate::components::{CapturesMouseInput, NeedsPaint, OnClick, OnInput, OnPaint, Position, Text};
use crate::types::{EventInput, MouseButton, MouseInput, Rect};
use crate::widgets::{Widget, WidgetSystem};
use crate::Result;
use cairo::cairo::Cairo;
use hecs::{Entity, World};

struct ButtonPressed;

pub struct ButtonSystem {
    on_paint: OnPaint,
    on_input: OnInput,
}

impl Default for ButtonSystem {
    fn default() -> Self {
        Self {
            on_paint: OnPaint::new(Self::on_paint),
            on_input: OnInput::new(Self::on_input)
        }
    }
}

impl ButtonSystem {
    fn on_paint(world: &World, entity: Entity, cr: &Cairo) {
        let mut query = world
            .query_one::<(Option<&ButtonPressed>, Option<(&Position, &Text)>)>(entity)
            .unwrap();

        let (button_pressed, position_text) = query.get().unwrap();

        if button_pressed.is_some() {
            cr.set_source_rgb(0.8, 0.0, 0.0);
            cr.translate(1.0, 1.0);
        } else {
            cr.set_source_rgb(1.0, 0.0, 0.0);
        }

        cr.paint();

        if let Some((&Position(pos), Text(ref text))) = position_text {
            let font_extents = cr.font_extents();
            let text_extents = cr.text_extents(text);
            cr.set_source_rgb(0.0, 0.0, 0.0)
                .move_to(
                    (pos.width - text_extents.width) / 2.0,
                    (pos.height + font_extents.height) / 2.0,
                )
                .show_text(text);
        }
    }

    fn on_input(world: &mut World, entity: Entity, input: EventInput) -> Result<()> {
        match input {
            EventInput::Mouse { input, x, y } => match input {
                MouseInput::ButtonDown {
                    button: MouseButton::Left,
                } => {
                    world
                        .insert(entity, (ButtonPressed, CapturesMouseInput, NeedsPaint))
                        .unwrap();
                }

                MouseInput::Move => {
                    let pressed = || {
                        let mut query = world
                            .query_one::<(&Position, Option<&ButtonPressed>, Option<&CapturesMouseInput>)>(entity)
                            .unwrap();

                        let (&Position(pos), button_pressed, captures_mouse_input) = query.get()?;
                        captures_mouse_input?;

                        let prev_pressed = button_pressed.is_some();
                        let pressed = Rect { x: 0.0, y: 0.0, ..pos }.contains(x, y);
                        if prev_pressed != pressed {
                            Some(pressed)
                        } else {
                            None
                        }
                    };

                    if let Some(pressed) = pressed() {
                        if pressed {
                            world.insert(entity, (ButtonPressed, NeedsPaint)).unwrap();
                        } else {
                            world.remove_one::<ButtonPressed>(entity).unwrap();
                            world.insert_one(entity, NeedsPaint).unwrap();
                        }
                    }
                }

                MouseInput::ButtonUp {
                    button: MouseButton::Left,
                } => {
                    if world.entity(entity).unwrap().get::<ButtonPressed>().is_some() {
                        world.remove::<(ButtonPressed, CapturesMouseInput)>(entity).unwrap();
                        world.insert_one(entity, NeedsPaint).unwrap();

                        let on_click = world.query_one::<&OnClick>(entity).unwrap().get().cloned();
                        if let Some(OnClick(on_click)) = on_click {
                            (on_click)(world, entity)?;
                        }
                    } else {
                        world.remove_one::<CapturesMouseInput>(entity).unwrap();
                    }
                }

                _ => (),
            },

            _ => (),
        }

        Ok(())
    }
}

impl WidgetSystem for ButtonSystem {
    type Widget = Button;
    type Components = (OnPaint, OnInput);

    fn components(&self) -> Self::Components {
        (self.on_paint.clone(), self.on_input.clone())
    }
}

pub struct Button;

impl Widget for Button {
    type System = ButtonSystem;
}
