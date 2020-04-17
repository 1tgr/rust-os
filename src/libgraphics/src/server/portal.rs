use crate::client;
use crate::frame_buffer::FrameBuffer;
use crate::server::screen::{PortalRef, ScreenBuffer, ScreenState};
use crate::types::Rect;
use alloc::sync::Arc;
use ecs::{ComponentStorage, System};
use os::{File, Mutex, Result, SharedMem};

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct ZIndex {
    index: isize,
    version: usize,
}

impl ZIndex {
    pub fn new() -> Self {
        Self {
            index: 0,
            version: client::alloc_id(),
        }
    }
}

impl Clone for ZIndex {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            version: client::alloc_id(),
        }
    }
}

pub struct ServerPortal {
    portal_ref: PortalRef,
    pos: Rect,
    prev_pos: Rect,
    z_index: ZIndex,
    frame_buffer: FrameBuffer,
    needs_paint: bool,
}

impl ServerPortal {
    pub fn new(
        e: &ComponentStorage,
        server2client: Arc<Mutex<File>>,
        id: usize,
        pos: Rect,
        shared_mem: SharedMem,
    ) -> Result<Self> {
        let z_index = e
            .components::<Self>()
            .map(|p| &p.z_index)
            .max()
            .cloned()
            .unwrap_or_else(|| ZIndex::new());

        let frame_buffer = FrameBuffer::from_shared_mem(pos.width, pos.height, shared_mem)?;

        Ok(Self {
            portal_ref: PortalRef {
                server2client,
                portal_id: id,
            },
            pos,
            prev_pos: pos,
            z_index,
            frame_buffer,
            needs_paint: true,
        })
    }

    pub fn move_to(&mut self, pos: Rect) {
        self.pos = pos;
    }

    pub fn invalidate(&mut self) {
        self.needs_paint = true;
    }

    fn as_screen_buffer(&mut self) -> ScreenBuffer {
        ScreenBuffer {
            pos: self.pos,
            frame_buffer_ptr: self.frame_buffer.as_mut_slice().as_mut_ptr(),
            portal_ref: self.portal_ref.clone(),
        }
    }
}

pub struct ServerPortalSystem {
    screen_state: Arc<Mutex<ScreenState>>,
    input_state: Arc<Mutex<Option<PortalRef>>>,
}

impl ServerPortalSystem {
    pub fn new(screen_state: Arc<Mutex<ScreenState>>, input_state: Arc<Mutex<Option<PortalRef>>>) -> Self {
        ServerPortalSystem {
            screen_state,
            input_state,
        }
    }
}

impl System for ServerPortalSystem {
    fn run(&mut self, e: &mut ComponentStorage) -> Result<()> {
        let any_deleted = e.deleted_components::<ServerPortal>().any(|_| true);
        let mut portals = e.components_mut::<ServerPortal>().collect::<Vec<_>>();
        portals.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        for portal in portals.iter_mut() {
            if portal.prev_pos != portal.pos {
                portal.frame_buffer.resize(portal.pos.width, portal.pos.height)?;
                portal.prev_pos = portal.pos;
                portal.needs_paint = true;
            }
        }

        *self.input_state.lock().unwrap() = portals.last().map(|p| p.portal_ref.clone());

        if any_deleted || portals.iter().any(|p| p.needs_paint) {
            let mut screen = self.screen_state.lock().unwrap();
            screen.buffers.clear();

            for portal in portals.iter_mut() {
                screen.buffers.push(portal.as_screen_buffer());
                portal.needs_paint = false;
            }

            screen.draw();
        }

        Ok(())
    }
}
