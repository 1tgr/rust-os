use crate::components::{BackColor, FontFace, OnPaint, Position, Text, TextColor};
use crate::widgets::WidgetSystem;
use cairo::Cairo;
use graphics_base::system::System;
use graphics_base::types::Color;
use graphics_base::Result;
use hecs::{Entity, World};

pub struct Label;

pub struct LabelSystem {
    on_paint: OnPaint,
}

impl LabelSystem {
    pub fn new() -> Self {
        Self {
            on_paint: OnPaint::new(Self::on_paint),
        }
    }

    fn on_paint(world: &World, entity: Entity, cr: &Cairo) {
        let mut query = world
            .query_one::<(
                &Position,
                Option<&BackColor>,
                Option<(&Text, Option<&FontFace>, Option<&TextColor>)>,
            )>(entity)
            .unwrap();

        if let Some((&Position(pos), back_color, text_and_style)) = query.get() {
            if let Some(&BackColor(Color { r, g, b })) = back_color {
                cr.set_source_rgb(r, g, b).paint();
            }

            if let Some((Text(ref text), font_face, text_color)) = text_and_style {
                let TextColor(Color { r, g, b }) = text_color.cloned().unwrap_or_else(|| TextColor::new(0.0, 0.0, 0.2));
                if let Some(FontFace(font_face)) = font_face {
                    cr.set_font_face(&font_face);
                }

                let font_extents = cr.font_extents();
                cr.set_source_rgb(r, g, b)
                    .move_to(
                        (pos.height - font_extents.height) / 2.0,
                        (pos.height + font_extents.height) / 2.0,
                    )
                    .show_text(text);
            }
        }
    }
}

impl WidgetSystem for LabelSystem {
    type Widget = Label;
    type Components = (OnPaint,);

    fn components(&self) -> Self::Components {
        (self.on_paint.clone(),)
    }
}

impl System for LabelSystem {
    fn run(&mut self, world: &mut World) -> Result<()> {
        WidgetSystem::run(self, world)
    }
}
