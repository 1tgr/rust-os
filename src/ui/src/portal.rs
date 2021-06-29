use crate::prelude::*;
use crate::render::{DrawTarget, RenderDb, RenderState, RenderStateObject};
use euclid::{Box2D, Size2D};
use raqote::SolidSource;
use ui_types::types::ScreenSpace;

#[derive(Clone, PartialEq)]
pub struct PortalRenderState {
    size: Size2D<f32, ScreenSpace>,
    color: SolidSource,
    children: Vec<Rc<dyn RenderStateObject>>,
}

impl HasProperty<Origin> for Portal {}
impl HasProperty<Size> for Portal {}
impl HasProperty<Color> for Portal {}
impl HasProperty<Text> for Portal {}

impl RenderState for PortalRenderState {
    fn build(db: &dyn RenderDb, widget_id: WidgetId) -> Self {
        Self {
            size: db.size(widget_id).unwrap_or_default().cast_unit(),
            color: db
                .color(widget_id)
                .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)),
            children: db
                .children(widget_id)
                .into_iter()
                .map(|widget_id| db.render_state(widget_id))
                .collect(),
        }
    }

    fn bounds(&self) -> Box2D<f32, ScreenSpace> {
        Box2D::from_size(self.size)
    }

    fn render_to(&self, target: &mut DrawTarget) {
        let Self {
            size: _,
            color,
            ref children,
        } = *self;
        target.clear(color);

        for child in children {
            child.render_to(target);
        }
    }
}

#[derive(Default, PartialEq, Hash)]
pub struct Portal;

impl Widget for Portal {
    type RenderState = PortalRenderState;
}
