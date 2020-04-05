#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Checkpoint { id: usize },

    KeyPress { portal_id: usize, code: char },
}
