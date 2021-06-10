use crate::button::ButtonStorage;
use crate::geometry::ScreenPoint;
use crate::input::{InputDb, InputStorage};
use crate::property::{PropertyDb, PropertyStorage};
use crate::render::RenderStorage;
use crate::widget::InternStorage;
use ui_types::types::MouseButton;

#[salsa::database(ButtonStorage, InputStorage, InternStorage, PropertyStorage, RenderStorage)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

impl Default for Database {
    fn default() -> Self {
        let mut db = Self {
            storage: Default::default(),
        };

        db.set_properties(Default::default());
        db.set_mouse_pos(ScreenPoint::zero());
        db.set_mouse_down_at(MouseButton::Left, None);
        db.set_mouse_down_at(MouseButton::Middle, None);
        db.set_mouse_down_at(MouseButton::Right, None);
        db
    }
}

impl Database {
    pub fn new() -> Self {
        Self::default()
    }
}
