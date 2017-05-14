#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Checkpoint {
        id: usize,
    },

    CreateWindow {
        id: usize,
        pos: Rect,
        shared_mem_handle: usize,
    },

    DestroyWindow {
        id: usize,
    },

    InvalidateWindow {
        id: usize,
    },

    MoveWindow {
        id: usize,
        pos: Rect,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Checkpoint {
        id: usize,
    },

    KeyPress {
        window_id: usize,
        code: char,
    }
}
