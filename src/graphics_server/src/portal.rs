use crate::compat::Mutex;
use crate::screen::{Screen, ScreenBuffer};
use alloc::sync::Arc;
use core::mem;
use graphics_base::frame_buffer::{AsSurfaceMut, FrameBuffer};
use graphics_base::system::{DeletedIndex, System};
use graphics_base::types::Rect;
use graphics_base::Result;
use hecs::World;

#[cfg(target_os = "rust_os")]
mod rust_os {
    use alloc::sync::Arc;
    use graphics_base::ipc;
    use graphics_base::types::{Event, EventInput};
    use graphics_base::Result;
    use os::{File, Mutex};

    #[derive(Clone)]
    pub struct PortalRef {
        pub server2client: Arc<Mutex<File>>,
        pub portal_id: usize,
    }

    impl PartialEq for PortalRef {
        fn eq(&self, other: &Self) -> bool {
            self.portal_id == other.portal_id && Arc::ptr_eq(&self.server2client, &other.server2client)
        }
    }

    impl PortalRef {
        pub fn send_input(&self, input: EventInput) -> Result<()> {
            let mut server2client = self.server2client.lock();
            ipc::send_message(
                &mut *server2client,
                &Event::Input {
                    portal_id: self.portal_id,
                    input,
                },
            )
        }
    }
}

#[cfg(not(target_os = "rust_os"))]
mod posix {
    use graphics_base::types::{Event, EventInput};
    use graphics_base::Result;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    #[derive(Clone)]
    pub struct PortalRef {
        pub portal_id: usize,
        pub events: Rc<RefCell<VecDeque<Event>>>,
    }

    impl PartialEq for PortalRef {
        fn eq(&self, other: &Self) -> bool {
            self.portal_id == other.portal_id && Rc::ptr_eq(&self.events, &other.events)
        }
    }

    impl PortalRef {
        pub fn send_input(&self, input: EventInput) -> Result<()> {
            let event = Event::Input {
                portal_id: self.portal_id,
                input,
            };

            self.events.borrow_mut().push_back(event);
            Ok(())
        }
    }
}

#[cfg(target_os = "rust_os")]
pub use rust_os::PortalRef;

#[cfg(not(target_os = "rust_os"))]
pub use posix::PortalRef;

pub struct ServerPortal {
    portal_ref: PortalRef,
    pos: Rect,
    prev_pos: Rect,
    z_index: usize,
    frame_buffer_id: usize,
    frame_buffer_size: (u16, u16),
    frame_buffer: Arc<FrameBuffer>,
    needs_paint: bool,
}

impl ServerPortal {
    pub fn new(
        world: &World,
        portal_ref: PortalRef,
        pos: Rect,
        frame_buffer_id: usize,
        frame_buffer_size: (u16, u16),
        frame_buffer: FrameBuffer,
    ) -> Self {
        let z_index = world
            .query::<&Self>()
            .iter()
            .map(|(_, portal)| &portal.z_index)
            .max()
            .cloned()
            .unwrap_or(0);

        Self {
            portal_ref,
            pos,
            prev_pos: pos,
            z_index,
            frame_buffer_id,
            frame_buffer_size,
            frame_buffer: Arc::new(frame_buffer),
            needs_paint: true,
        }
    }
}

impl ServerPortal {
    pub fn move_to(&mut self, pos: Rect) {
        self.pos = pos;
    }

    pub fn draw(&mut self, frame_buffer_id: usize, frame_buffer_size: (u16, u16), frame_buffer: FrameBuffer) -> usize {
        self.frame_buffer_size = frame_buffer_size;
        self.frame_buffer = Arc::new(frame_buffer);
        self.needs_paint = true;
        mem::replace(&mut self.frame_buffer_id, frame_buffer_id)
    }
}

impl ServerPortal {
    fn as_screen_buffer(&self) -> ScreenBuffer {
        ScreenBuffer {
            pos: self.pos,
            frame_buffer_size: self.frame_buffer_size,
            frame_buffer: Arc::downgrade(&self.frame_buffer),
            portal_ref: self.portal_ref.clone(),
        }
    }
}

pub struct ServerPortalSystem<S> {
    screen: Arc<Mutex<Screen<S>>>,
    input_state: Arc<Mutex<Option<PortalRef>>>,
    deleted_index: DeletedIndex<()>,
}

impl<S> ServerPortalSystem<S> {
    pub fn new(screen: Arc<Mutex<Screen<S>>>, input_state: Arc<Mutex<Option<PortalRef>>>) -> Self {
        ServerPortalSystem {
            screen,
            input_state,
            deleted_index: DeletedIndex::new(),
        }
    }
}

impl<S> System for ServerPortalSystem<S>
where
    S: AsSurfaceMut,
{
    fn run(&mut self, world: &mut World) -> Result<()> {
        let mut portals_borrow = world.query::<&mut ServerPortal>();
        let mut portals = portals_borrow.iter().map(|(_, portal)| portal).collect::<Vec<_>>();
        portals.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        for portal in portals.iter_mut() {
            if portal.prev_pos != portal.pos {
                portal.prev_pos = portal.pos;
                portal.needs_paint = true;
            }
        }

        *self.input_state.lock() = portals.last().map(|p| p.portal_ref.clone());

        let deleted_entities = self
            .deleted_index
            .update(world.query::<()>().with::<ServerPortal>().iter());

        if !deleted_entities.is_empty() || portals.iter().any(|p| p.needs_paint) {
            self.screen
                .lock()
                .update_buffers(portals.iter_mut().rev().map(|portal| {
                    portal.needs_paint = false;
                    portal.as_screen_buffer()
                }));
        }

        Ok(())
    }
}
