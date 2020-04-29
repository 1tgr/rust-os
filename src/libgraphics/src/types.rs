#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Checkpoint {
        id: usize,
    },

    CreatePortal {
        id: usize,
        pos: Rect,
        size: (u16, u16),
        frame_buffer_id: usize,
        shared_mem_handle: usize,
    },

    DestroyPortal {
        id: usize,
    },

    DrawPortal {
        id: usize,
        size: (u16, u16),
        frame_buffer_id: usize,
        shared_mem_handle: usize,
    },

    MovePortal {
        id: usize,
        pos: Rect,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MouseInput {
    ButtonDown { button: MouseButton },
    ButtonUp { button: MouseButton },
    Move,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventInput {
    KeyPress { code: char },
    Mouse { x: f64, y: f64, input: MouseInput },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Checkpoint { id: usize },
    ReuseFrameBuffer { frame_buffer_id: usize },
    Input { portal_id: usize, input: EventInput },
}
