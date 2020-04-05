use crate::client;
use crate::frame_buffer::FrameBuffer;
use crate::types::Rect;
use alloc::sync::Arc;
use cairo::bindings::*;
use cairo::cairo::Cairo;
use cairo::surface::CairoSurface;
use ecs::{ComponentStorage, System};
use os::{File, Mutex, OSMem, Result, SharedMem};

#[derive(Clone)]
pub struct PortalRef {
    pub server2client: Arc<Mutex<File>>,
    pub id: usize,
}

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

        let frame_buffer = FrameBuffer::new(pos.width, pos.height, shared_mem)?;

        Ok(Self {
            portal_ref: PortalRef { server2client, id },
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
}

pub struct ServerPortalSystem {
    lfb_mem: OSMem,
    input_state: Arc<Mutex<Option<PortalRef>>>,
}

impl ServerPortalSystem {
    pub fn new(lfb_mem: OSMem, input_state: Arc<Mutex<Option<PortalRef>>>) -> Self {
        ServerPortalSystem { lfb_mem, input_state }
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
            let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
            let surface = CairoSurface::from_slice(&mut *self.lfb_mem, CAIRO_FORMAT_ARGB32, 800, 600, stride);
            let cr = Cairo::new(surface);
            cr.set_source_rgb(0.0, 0.0, 0.5);
            cr.paint();

            for portal in portals {
                let ServerPortal {
                    pos: Rect { x, y, .. },
                    ref mut frame_buffer,
                    ref mut needs_paint,
                    ..
                } = *portal;

                let surface = frame_buffer.as_surface();
                cr.set_source_surface(&surface, x, y);
                cr.paint();
                *needs_paint = false;
            }
        }

        Ok(())
    }
}
