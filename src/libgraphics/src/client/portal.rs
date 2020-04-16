use crate::client::ClientPipe;
use crate::frame_buffer::FrameBuffer;
use crate::types::{Command, EventInput, Rect};
use crate::{client, Event};
use alloc::rc::Rc;
use cairo::cairo::Cairo;
use core::mem;
use ecs::{ComponentStorage, System};
use os::{Result, SharedMem};

pub trait Handler {
    fn on_paint(&self, cr: &Cairo);
    fn on_input(&self, e: &mut ComponentStorage, inputs: Vec<EventInput>);
}

pub struct ClientPortal {
    pos: Rect,
    handler: Rc<dyn Handler>,
    prev_pos: Rect,
    id: usize,
    frame_buffer: FrameBuffer,
    needs_paint: bool,
    input: Vec<EventInput>,
}

impl ClientPortal {
    pub fn new<H>(e: &mut ComponentStorage, pos: Rect, handler: H) -> Result<Self>
    where
        H: Handler + 'static,
    {
        let id = client::alloc_id();
        let shared_mem = SharedMem::new(true)?;

        let command = Command::CreatePortal {
            id,
            pos,
            shared_mem_handle: shared_mem.handle().get(),
        };

        for pipe in e.components_mut::<ClientPipe>() {
            pipe.send_command(&command)?;
        }

        let frame_buffer = FrameBuffer::from_shared_mem(pos.width, pos.height, shared_mem)?;

        Ok(Self {
            pos,
            handler: Rc::new(handler),
            prev_pos: pos,
            id,
            frame_buffer,
            needs_paint: true,
            input: Vec::new(),
        })
    }

    pub fn move_to(&mut self, pos: Rect) {
        self.pos = pos;
        self.invalidate();
    }

    pub fn invalidate(&mut self) {
        self.needs_paint = true;
    }

    pub fn dispatch_event(&mut self, event: &Event) {
        match *event {
            Event::Input { portal_id, ref input } if portal_id == self.id => {
                self.input.push(input.clone());
            }
            _ => (),
        }
    }
}

pub struct ClientPortalSystem;

impl System for ClientPortalSystem {
    fn run(&mut self, e: &mut ComponentStorage) -> Result<()> {
        let mut input_handlers = Vec::new();
        for portal in e.components_mut::<ClientPortal>() {
            if !portal.input.is_empty() {
                input_handlers.push((portal.handler.clone(), mem::replace(&mut portal.input, Vec::new())));
            }
        }

        for (handler, v) in input_handlers {
            handler.on_input(e, v);
        }

        let mut commands = Vec::new();
        for portal in e.components_mut::<ClientPortal>() {
            if portal.prev_pos != portal.pos {
                commands.push(Command::MovePortal {
                    id: portal.id,
                    pos: portal.pos,
                });

                portal.prev_pos = portal.pos;
                portal.needs_paint = true;
            }

            if portal.needs_paint {
                let cr = portal.frame_buffer.as_surface().into_cairo();
                cr.set_source_rgb(1.0, 1.0, 1.5);
                cr.paint();
                portal.handler.on_paint(&cr);
                portal.needs_paint = false;
                commands.push(Command::InvalidatePortal { id: portal.id });
            }
        }

        for portal in e.deleted_components::<ClientPortal>() {
            commands.push(Command::DestroyPortal { id: portal.id });
        }

        for pipe in e.components_mut::<ClientPipe>() {
            for command in commands.iter() {
                pipe.send_command(command)?;
            }
        }

        Ok(())
    }
}
