use crate::geometry::{ObjectSize, ScreenTransform};
use crate::input::InputDb;
use crate::path::Path;
use crate::prelude::*;
use crate::property::PropertyDb;
use crate::render::{DrawTarget, RenderDb, RenderState};
use euclid::Box2D;
use fontdue::layout::{HorizontalAlign, LayoutSettings, VerticalAlign};
use fontdue::{Font, FontSettings};
use lazy_static::lazy_static;
use raqote::{DrawOptions, SolidSource, Source};
use sprite::Sprite;
use ui_types::types::{MouseButton, ScreenSpace};

lazy_static! {
    static ref FONT: Font = {
        let font = include_bytes!("Vera.ttf");
        Font::from_bytes(font as &[u8], FontSettings::default()).unwrap()
    };
}

#[derive(Clone, PartialEq)]
pub struct ButtonRenderState {
    color: SolidSource,
    text_color: SolidSource,
    size: ObjectSize,
    text: Option<Rc<String>>,
    transform: ScreenTransform,
}

impl HasProperty<Color> for Button {}
impl HasProperty<Size> for Button {}
impl HasProperty<Text> for Button {}
impl HasProperty<TextColor> for Button {}
impl HasProperty<Transform> for Button {}

impl RenderState for ButtonRenderState {
    fn build(db: &dyn RenderDb, widget_id: WidgetId) -> Self {
        let mut color = db
            .color(widget_id)
            .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(255, 255, 255, 255));

        let mut text_color = db
            .text_color(widget_id)
            .unwrap_or_else(|| SolidSource::from_unpremultiplied_argb(255, 0, 0, 0));

        let down = db.button_down(widget_id);
        if down {
            color.r = (color.r * 3) / 4;
            color.g = (color.g * 3) / 4;
            color.b = (color.b * 3) / 4;
            text_color.r = (text_color.r * 3) / 4;
            text_color.g = (text_color.g * 3) / 4;
            text_color.b = (text_color.b * 3) / 4;
        }

        Self {
            color,
            text_color,
            size: db.size(widget_id).unwrap_or_default(),
            text: db.text(widget_id),
            transform: db.screen_transform(widget_id),
        }
    }

    fn bounds(&self) -> Box2D<f32, ScreenSpace> {
        Path::Rect(Box2D::from_size(self.size))
            .transform(&self.transform)
            .bounds()
    }

    fn render_to(&self, target: &mut DrawTarget) {
        let Self {
            color,
            text_color,
            size,
            ref text,
            ref transform,
        } = *self;

        let path = Path::Rect(Box2D::from_size(size)).transform(&transform);
        let rq_path = path.to_rq_path();
        target.fill(&rq_path, &Source::Solid(color), &DrawOptions::new());

        if let Some(text) = text {
            let bounds = transform.outer_transformed_box(&Box2D::from_size(size));
            let mut settings = LayoutSettings::default();
            settings.x = bounds.min.x;
            settings.y = bounds.min.y;
            settings.max_width = Some(bounds.width());
            settings.max_height = Some(bounds.height());
            settings.horizontal_align = HorizontalAlign::Center;
            settings.vertical_align = VerticalAlign::Middle;
            Sprite::from_draw_target(target).draw_text(&FONT, 12.0, text_color, &settings, &text);
        }
    }
}

#[derive(Default, PartialEq, Hash)]
pub struct Button;

impl Widget for Button {
    type RenderState = ButtonRenderState;
}

#[salsa::query_group(ButtonStorage)]
pub trait ButtonDb: PropertyDb + InputDb {
    fn button_down(&self, widget_id: WidgetId) -> bool;
}

fn button_down(db: &dyn ButtonDb, widget_id: WidgetId) -> bool {
    if let Some(at) = db.mouse_down_at(MouseButton::Left) {
        let path = db.screen_path(widget_id);
        let mouse_pos = db.mouse_pos();
        path.contains(0.0, at) && path.contains(0.0, mouse_pos)
    } else {
        false
    }
}
