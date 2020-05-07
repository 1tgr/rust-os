use crate::components::{BackColor, OnPaint, Position, Text, TextColor};
use crate::types::Color;
use crate::widgets::WidgetSystem;
use cairo::cairo::Cairo;
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
            .query_one::<(&Position, Option<&BackColor>, Option<(&Text, Option<&TextColor>)>)>(entity)
            .unwrap();

        if let Some((&Position(pos), back_color, text_and_text_color)) = query.get() {
            if let Some(&BackColor(Color { r, g, b })) = back_color {
                cr.set_source_rgb(r, g, b).paint();
            }

            if let Some((Text(ref text), text_color)) = text_and_text_color {
                let TextColor(Color { r, g, b }) = text_color.cloned().unwrap_or_else(|| TextColor::new(0.0, 0.0, 0.2));
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
