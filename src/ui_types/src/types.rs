use euclid::{Rect, Size2D};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScreenSpace {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatePortal {
    pub id: usize,
    pub pos: Rect<f32, ScreenSpace>,
    pub size: Size2D<i32, ScreenSpace>,
    pub frame_buffer_id: usize,
    pub shared_mem_handle: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DestroyPortal {
    pub id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DrawPortal {
    pub id: usize,
    pub size: Size2D<i32, ScreenSpace>,
    pub frame_buffer_id: usize,
    pub shared_mem_handle: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovePortal {
    pub id: usize,
    pub pos: Rect<f32, ScreenSpace>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Checkpoint(Checkpoint),
    CreatePortal(CreatePortal),
    DestroyPortal(DestroyPortal),
    DrawPortal(DrawPortal),
    MovePortal(MovePortal),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MouseInputInfo {
    pub x: f32,
    pub y: f32,
    pub screen_x: f32,
    pub screen_y: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventInput {
    KeyPress { code: char },
    MouseButtonDown { info: MouseInputInfo, button: MouseButton },
    MouseButtonUp { info: MouseInputInfo, button: MouseButton },
    MouseMove { info: MouseInputInfo },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Checkpoint { id: usize },
    ReuseFrameBuffer { frame_buffer_id: usize },
    Input { portal_id: usize, input: EventInput },
}
