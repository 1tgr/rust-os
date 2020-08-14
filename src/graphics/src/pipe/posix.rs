use crate::pipe;
use alloc::boxed::Box;
use alloc::sync::Arc;
use cairo::bindings::CAIRO_FORMAT_ARGB32;
use core::slice;
use graphics_base::frame_buffer::FrameBuffer;
use graphics_base::system::System;
use graphics_base::types::{Command, Event, EventInput};
use graphics_base::Result;
use graphics_server::{PortalRef, Screen, ServerPortal, ServerPortalSystem};
use hashbrown::HashMap;
use hecs::{Entity, World};
use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use std::cell::RefCell;
use std::char;
use std::collections::VecDeque;
use std::process;
use std::rc::Rc;
use std::sync::Mutex;
use std::time::Duration;

struct InputCallback(Arc<Mutex<Option<PortalRef>>>);

impl minifb::InputCallback for InputCallback {
    fn add_char(&mut self, uni_char: u32) {
        if let &Some(PortalRef { portal_id, ref events }) = &*self.0.lock().unwrap() {
            let code = char::from_u32(uni_char).unwrap();
            events.borrow_mut().push_back(Event::Input {
                portal_id,
                input: EventInput::KeyPress { code },
            });
        }
    }
}

pub struct ClientPipe {
    window: Window,
    buffer: Box<[u32]>,
    portals_by_id: HashMap<usize, Entity>,
    world: World,
    screen: Arc<Mutex<Screen<&'static mut [u8]>>>,
    system: ServerPortalSystem<&'static mut [u8]>,
    events: Rc<RefCell<VecDeque<Event>>>,
}

impl ClientPipe {
    pub fn new() -> Self {
        let input_state = Arc::new(Mutex::new(None));
        let mut window = Window::new("libgraphics", 800, 600, WindowOptions::default()).unwrap();
        window.limit_update_rate(Some(Duration::from_micros(16600)));
        window.set_input_callback(Box::new(InputCallback(input_state.clone())));

        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        let byte_len = stride * 600;
        let mut buffer = vec![0; byte_len / 4].into_boxed_slice();
        let aliased_buffer = unsafe { slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, byte_len) };
        let screen = Arc::new(Mutex::new(Screen::new((800, 600), aliased_buffer)));
        let system = ServerPortalSystem::new(screen.clone(), input_state);

        Self {
            window,
            buffer,
            portals_by_id: HashMap::new(),
            world: World::new(),
            screen,
            system,
            events: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn send_command(&mut self, command: &Command) -> Result<()> {
        match *command {
            Command::Checkpoint { id: _id } => (),

            Command::CreatePortal {
                id,
                pos,
                size,
                frame_buffer_id,
                shared_mem_handle,
            } => {
                let portal_ref = PortalRef {
                    portal_id: id,
                    events: self.events.clone(),
                };

                let frame_buffer = FrameBuffer::from_raw(size, shared_mem_handle)?;
                let portal = ServerPortal::new(&self.world, portal_ref, pos, frame_buffer_id, size, frame_buffer);
                let entity = self.world.spawn((portal,));
                self.portals_by_id.insert(id, entity);
            }

            Command::DestroyPortal { id } => {
                if let Some(entity) = self.portals_by_id.remove(&id) {
                    self.world.despawn(entity).unwrap();
                }
            }

            Command::DrawPortal {
                id,
                size,
                frame_buffer_id,
                shared_mem_handle,
            } => {
                if let Some(entity) = self.portals_by_id.get(&id).copied() {
                    let frame_buffer = FrameBuffer::from_raw(size, shared_mem_handle)?;

                    let frame_buffer_id =
                        self.world
                            .get_mut::<ServerPortal>(entity)
                            .unwrap()
                            .draw(frame_buffer_id, size, frame_buffer);

                    self.events
                        .borrow_mut()
                        .push_back(Event::ReuseFrameBuffer { frame_buffer_id });
                }
            }

            Command::MovePortal { id, pos } => {
                if let Some(entity) = self.portals_by_id.get(&id).copied() {
                    self.world.get_mut::<ServerPortal>(entity).unwrap().move_to(pos);
                }
            }
        }

        Ok(())
    }

    pub fn wait_for_event(&mut self) -> Result<Event> {
        loop {
            if !self.window.is_open() {
                process::exit(0);
            }

            if let Some(event) = self.events.borrow_mut().pop_front() {
                return Ok(event);
            }

            self.system.run(&mut self.world)?;
            self.window.update_with_buffer(&*self.buffer, 800, 600).unwrap();

            if let Some((x, y)) = self.window.get_mouse_pos(MouseMode::Discard) {
                let x = x as u16;
                let y = y as u16;

                let mut buttons = [false; 3];
                for (down, &button) in buttons
                    .iter_mut()
                    .zip([MouseButton::Left, MouseButton::Middle, MouseButton::Right].iter())
                {
                    *down = self.window.get_mouse_down(button);
                }

                self.screen.lock().update_mouse_state(x, y, 0, buttons)?;
            }
        }
    }

    pub fn checkpoint(&mut self) -> Result<usize> {
        let id = pipe::alloc_id();
        self.send_command(&Command::Checkpoint { id })?;
        Ok(id)
    }
}
