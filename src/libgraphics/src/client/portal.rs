use crate::client::ClientPipe;
use crate::frame_buffer::FrameBuffer;
use crate::types::{Command, Rect};
use crate::{client, Event};
use alloc::rc::Rc;
use cairo::cairo::Cairo;
use core::mem;
use ecs::{ComponentStorage, System};
use os::{Result, SharedMem};

pub struct ClientPortal {
    pos: Rect,
    on_paint: Box<dyn Fn(&Cairo) -> ()>,
    on_key_press: Rc<dyn Fn(&mut ComponentStorage, char) -> ()>,
    prev_pos: Rect,
    id: usize,
    frame_buffer: FrameBuffer,
    needs_paint: bool,
    waiting_keys: Vec<char>,
}

impl ClientPortal {
    pub fn new<OnPaint, OnKeyPress>(
        e: &mut ComponentStorage,
        pos: Rect,
        on_paint: OnPaint,
        on_key_press: OnKeyPress,
    ) -> Result<Self>
    where
        OnPaint: Fn(&Cairo) -> () + 'static,
        OnKeyPress: Fn(&mut ComponentStorage, char) -> () + 'static,
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

        let frame_buffer = FrameBuffer::new(pos.width, pos.height, shared_mem)?;

        Ok(Self {
            pos,
            on_paint: Box::new(on_paint),
            on_key_press: Rc::new(on_key_press),
            prev_pos: pos,
            id,
            frame_buffer,
            needs_paint: true,
            waiting_keys: Vec::new(),
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
            Event::KeyPress { portal_id, code } if portal_id == self.id => {
                self.waiting_keys.push(code);
            }
            _ => (),
        }
    }
}

pub struct ClientPortalSystem;

impl System for ClientPortalSystem {
    fn run(&mut self, e: &mut ComponentStorage) -> Result<()> {
        let mut waiting_keys = Vec::new();
        for portal in e.components_mut::<ClientPortal>() {
            if !portal.waiting_keys.is_empty() {
                waiting_keys.push((
                    portal.on_key_press.clone(),
                    mem::replace(&mut portal.waiting_keys, Vec::new()),
                ));
            }
        }

        for (f, codes) in waiting_keys {
            for code in codes {
                f(e, code);
            }
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
                let surface = portal.frame_buffer.as_surface();
                let cr = Cairo::new(surface);
                cr.set_source_rgb(1.0, 1.0, 1.5);
                cr.paint();
                (portal.on_paint)(&cr);
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
