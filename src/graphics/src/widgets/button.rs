use crate::components::{
    BackColor, CapturesMouseInput, FontFace, NeedsPaint, OnClick, OnInput, OnPaint, Position, Text, TextColor,
};
use crate::widgets::WidgetSystem;
use cairo::Cairo;
use graphics_base::system::System;
use graphics_base::types::{Color, EventInput, MouseButton, Rect};
use graphics_base::Result;
use hecs::{Entity, World};

pub struct Button;

struct ButtonPressed;

pub struct ButtonSystem {
    on_paint: OnPaint,
    on_input: OnInput,
}

impl ButtonSystem {
    pub fn new() -> Self {
        Self {
            on_paint: OnPaint::new(Self::on_paint),
            on_input: OnInput::new(Self::on_input),
        }
    }

    fn on_paint(world: &World, entity: Entity, cr: &Cairo) {
        let mut query = world
            .query_one::<(
                Option<&ButtonPressed>,
                Option<&BackColor>,
                Option<(&Position, Option<&FontFace>, Option<&TextColor>, Option<&Text>)>,
            )>(entity)
            .unwrap();

        let (button_pressed, back_color, position_text) = query.get().unwrap();
        let BackColor(back_color) = back_color.cloned().unwrap_or_else(|| BackColor::new(0.76, 0.74, 0.96));

        let Color { r, g, b } = if button_pressed.is_some() {
            back_color * 0.8
        } else {
            back_color
        };

        cr.set_source_rgb(r, g, b).paint();

        if let Some((&Position(pos), font_face, text_color, text)) = position_text {
            let TextColor(Color { r, g, b }) = text_color.cloned().unwrap_or_else(|| TextColor::new(0.0, 0.0, 0.2));
            cr.set_source_rgb(r, g, b)
                .rectangle(0.0, 0.0, pos.width, pos.height)
                .stroke();

            if button_pressed.is_some() {
                cr.translate(1.0, 1.0);
            }

            if let Some(Text(ref text)) = text {
                if let Some(FontFace(font_face)) = font_face {
                    cr.set_font_face(&font_face);
                }

                let font_extents = cr.font_extents();
                let text_extents = cr.text_extents(text);
                cr.move_to(
                    (pos.width - text_extents.width) / 2.0,
                    (pos.height + font_extents.height) / 2.0,
                )
                .show_text(text);
            }
        }
    }

    fn on_input(world: &mut World, entity: Entity, input: EventInput) -> Result<()> {
        match input {
            EventInput::MouseButtonDown {
                button: MouseButton::Left,
                ..
            } => {
                world.insert(entity, (ButtonPressed, NeedsPaint)).unwrap();
            }

            EventInput::MouseMove { info } => {
                let pressed = || {
                    let mut query = world
                        .query_one::<(&Position, Option<&ButtonPressed>, Option<&CapturesMouseInput>)>(entity)
                        .unwrap();

                    let (&Position(pos), button_pressed, captures_mouse_input) = query.get()?;
                    captures_mouse_input?;

                    let prev_pressed = button_pressed.is_some();
                    let pressed = Rect { x: 0.0, y: 0.0, ..pos }.contains(info.x, info.y);
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

            EventInput::MouseButtonUp {
                button: MouseButton::Left,
                ..
            } => {
                if let Ok(ButtonPressed) = world.remove_one(entity) {
                    world.insert_one(entity, NeedsPaint).unwrap();

                    let on_click = world.query_one::<&OnClick>(entity).unwrap().get().cloned();
                    if let Some(OnClick(on_click)) = on_click {
                        (on_click)(world, entity)?;
                    }
                }
            }

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

impl System for ButtonSystem {
    fn run(&mut self, world: &mut World) -> Result<()> {
        WidgetSystem::run(self, world)
    }
}
