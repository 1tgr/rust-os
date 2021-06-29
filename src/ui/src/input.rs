use crate::geometry::ScreenPoint;
use ui_types::types::MouseButton;

#[salsa::query_group(InputStorage)]
pub trait InputDb: salsa::Database {
    #[salsa::input]
    fn mouse_pos(&self) -> ScreenPoint;

    #[salsa::input]
    fn mouse_down_at(&self, button: MouseButton) -> Option<ScreenPoint>;
}
