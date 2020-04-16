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
        shared_mem_handle: usize,
    },

    DestroyPortal {
        id: usize,
    },

    InvalidatePortal {
        id: usize,
    },

    MovePortal {
        id: usize,
        pos: Rect,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventInput {
    KeyPress { code: char },
    MouseDown { x: f64, y: f64, button: MouseButton },
    MouseUp { x: f64, y: f64, button: MouseButton },
    MouseMove { x: f64, y: f64 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Checkpoint { id: usize },
    Input { portal_id: usize, input: EventInput },
}
