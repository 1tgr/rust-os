use crate::geometry::{ObjectSize, ScreenTransform};
use crate::path::Path;
use crate::prelude::*;
use crate::render::{DrawTarget, RenderDb, RenderState, RenderStateObject};
use alloc::rc::Rc;
use euclid::Box2D;
use raqote::{BlendMode, DrawOptions, SolidSource, Source};
use ui_types::types::ScreenSpace;

#[derive(Clone, PartialEq)]
pub struct PanelRenderState {
    transform: ScreenTransform,
    size: ObjectSize,
    color: SolidSource,
    children: Vec<Rc<dyn RenderStateObject>>,
}

impl HasProperty<Color> for Panel {}
impl HasProperty<Size> for Panel {}
impl HasProperty<Transform> for Panel {}

impl RenderState for PanelRenderState {
    fn build(db: &dyn RenderDb, widget_id: WidgetId) -> Self {
        Self {
            transform: db.screen_transform(widget_id),
            size: db.size(widget_id).unwrap_or_default(),
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
        Path::Rect(Box2D::from_size(self.size))
            .transform(&self.transform)
            .bounds()
    }

    fn render_to(&self, target: &mut DrawTarget) {
        let Self {
            ref transform,
            size,
            color,
            ref children,
        } = *self;

        let mut options = DrawOptions::new();
        if color.a == 255 {
            options.blend_mode = BlendMode::Src;
        }

        let src = Source::Solid(color);

        let path = Path::Rect(Box2D::from_size(size)).transform(&transform);
        match &path {
            Path::Rect(rect) => {
                target.fill_rect(rect.min.x, rect.min.y, rect.width(), rect.height(), &src, &options);
            }
            Path::Path(path) => {
                target.fill(path, &src, &options);
            }
        }

        if !children.is_empty() {
            if let Some(int_r) = path.as_int_rect() {
                target.push_clip_rect(int_r.to_untyped());
            } else {
                let rq_path = path.to_rq_path();
                target.push_clip(&rq_path);
            }

            for child in children {
                child.render_to(target);
            }

            target.pop_clip();
        }
    }
}

#[derive(Default, PartialEq, Hash)]
pub struct Panel;

impl Widget for Panel {
    type RenderState = PanelRenderState;
}
