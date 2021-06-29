#![allow(clippy::transmute_ptr_to_ref)]
use crate::button::ButtonDb;
use crate::widget::WidgetId;
use alloc::rc::Rc;
use euclid::Box2D;
use mopa::Any;
use ui_types::types::ScreenSpace;

pub type DrawTarget<'a> = raqote::DrawTarget<&'a mut [u32]>;

pub trait RenderStateObject: Any {
    fn eq(&self, other: &dyn RenderStateObject) -> bool;
    fn bounds(&self) -> Box2D<f32, ScreenSpace>;
    fn render_to(&self, target: &mut DrawTarget);
}

mopafy!(RenderStateObject, core = core, alloc = alloc);

impl PartialEq for dyn RenderStateObject {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

pub trait RenderState: PartialEq {
    fn build(db: &dyn RenderDb, widget_id: WidgetId) -> Self;
    fn bounds(&self) -> Box2D<f32, ScreenSpace>;
    fn render_to(&self, target: &mut DrawTarget);
}

impl<T> RenderStateObject for T
where
    T: RenderState + 'static,
{
    fn eq(&self, other: &dyn RenderStateObject) -> bool {
        other.downcast_ref().map_or(false, |other| self == other)
    }

    fn bounds(&self) -> Box2D<f32, ScreenSpace> {
        self.bounds()
    }

    fn render_to(&self, target: &mut DrawTarget) {
        self.render_to(target)
    }
}

#[salsa::query_group(RenderStorage)]
pub trait RenderDb: ButtonDb {
    fn render_state(&self, widget_id: WidgetId) -> Rc<dyn RenderStateObject>;
}

fn render_state(db: &dyn RenderDb, widget_id: WidgetId) -> Rc<dyn RenderStateObject> {
    db.archetype(widget_id).unwrap().render_state(db, widget_id)
}
