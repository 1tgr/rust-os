use core::ops::Mul;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Mul<f64> for Color {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self {
        let Self { r, g, b } = self;
        Self {
            r: r * rhs,
            g: g * rhs,
            b: b * rhs,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    KeyPress {
        code: char,
    },
    Mouse {
        x: f64,
        y: f64,
        screen_x: f64,
        screen_y: f64,
        input: MouseInput,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Checkpoint { id: usize },
    ReuseFrameBuffer { frame_buffer_id: usize },
    Input { portal_id: usize, input: EventInput },
}
