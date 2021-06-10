#![allow(clippy::transmute_ptr_to_ref)]
use crate::render::{RenderDb, RenderState, RenderStateObject};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec::Vec;
use core::hash::Hash;
use mopa::Any;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum WidgetPathSegment {
    Ordinal(usize),
    Key(String),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetId(salsa::InternId);

impl salsa::InternKey for WidgetId {
    fn from_intern_id(v: salsa::InternId) -> Self {
        Self(v)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[salsa::query_group(InternStorage)]
pub trait InternDb {
    #[salsa::interned]
    fn intern_widget_path(&self, path: Vec<WidgetPathSegment>) -> WidgetId;
}

pub trait WidgetObject: Any {
    fn eq(&self, other: &dyn WidgetObject) -> bool;
    fn render_state(&self, db: &dyn RenderDb, widget_id: WidgetId) -> Rc<dyn RenderStateObject>;
}

mopafy!(WidgetObject, core = core, alloc = alloc);

impl PartialEq for dyn WidgetObject {
    fn eq(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

pub trait Widget: PartialEq + Hash {
    type RenderState: RenderState;
}

impl<T> WidgetObject for T
where
    T: Widget + 'static,
{
    fn eq(&self, other: &dyn WidgetObject) -> bool {
        other.downcast_ref().map_or(false, |other| self == other)
    }

    fn render_state(&self, db: &dyn RenderDb, widget_id: WidgetId) -> Rc<dyn RenderStateObject> {
        Rc::new(T::RenderState::build(db, widget_id))
    }
}
