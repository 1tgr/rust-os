use crate::components::{OnPaint, Position, Text};
use crate::widgets::WidgetSystem;
use cairo::cairo::Cairo;
use hecs::{Entity, World};

pub struct Label;

pub(super) struct LabelSystem {
    on_paint: OnPaint,
}

impl LabelSystem {
    pub fn new() -> Self {
        Self {
            on_paint: OnPaint::new(Self::on_paint),
        }
    }

    fn on_paint(world: &World, entity: Entity, cr: &Cairo) {
        let mut query = world.query_one::<(&Position, &Text)>(entity).unwrap();

        if let Some((&Position(pos), Text(ref text))) = query.get() {
            let font_extents = cr.font_extents();
            cr.set_source_rgb(0.0, 0.0, 0.0)
                .move_to(0.0, (pos.height + font_extents.height) / 2.0)
                .show_text(text);
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
