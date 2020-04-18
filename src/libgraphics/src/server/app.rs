use crate::frame_buffer::FrameBuffer;
use crate::ipc;
use crate::server::portal::{ServerPortal, ServerPortalSystem};
use crate::server::screen::{PortalRef, ScreenState};
use crate::system::System;
use crate::types::{Command, Event, EventInput, MouseButton};
use crate::Result;
use alloc::sync::Arc;
use cairo::bindings::*;
use hashbrown::HashMap;
use hecs::{Entity, World};
use os::{File, Mutex, OSMem, Process, SharedMem};

struct MouseState {
    screen_width: u16,
    screen_height: u16,
    cursor_x: u16,
    cursor_y: u16,
    left: bool,
    middle: bool,
    right: bool,
}

impl MouseState {
    fn update(
        &mut self,
        dx: i16,
        dy: i16,
        _dw: i8,
        left: bool,
        middle: bool,
        right: bool,
    ) -> (f64, f64, Vec<EventInput>) {
        let x = ((self.cursor_x as i32 + dx as i32).max(0) as u16).min(self.screen_width - 1);
        let y = ((self.cursor_y as i32 + dy as i32).max(0) as u16).min(self.screen_height - 1);
        let mut inputs = Vec::new();

        if x != self.cursor_x || y != self.cursor_y {
            inputs.push(EventInput::MouseMove {
                x: x as f64,
                y: y as f64,
            });
        }

        if left && !self.left {
            inputs.push(EventInput::MouseDown {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Left,
            });
        } else if !left && self.left {
            inputs.push(EventInput::MouseUp {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Left,
            });
        }

        if middle && !self.middle {
            inputs.push(EventInput::MouseDown {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Middle,
            });
        } else if !middle && self.middle {
            inputs.push(EventInput::MouseUp {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Middle,
            });
        }

        if right && !self.right {
            inputs.push(EventInput::MouseDown {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Right,
            });
        } else if !right && self.right {
            inputs.push(EventInput::MouseUp {
                x: x as f64,
                y: y as f64,
                button: MouseButton::Right,
            });
        }

        self.cursor_x = x;
        self.cursor_y = y;
        self.left = left;
        self.middle = middle;
        self.right = right;
        (x as f64, y as f64, inputs)
    }
}

#[derive(Clone)]
pub struct ServerInput {
    keyboard_focus: Arc<Mutex<Option<PortalRef>>>,
    mouse_state: Arc<Mutex<MouseState>>,
    screen_state: Arc<Mutex<ScreenState>>,
}

impl ServerInput {
    fn new(lfb: FrameBuffer) -> Result<Self> {
        let screen_width = lfb.width_i();
        let screen_height = lfb.height_i();
        let cursor_x = screen_width / 2;
        let cursor_y = screen_height / 2;

        let mouse_state = MouseState {
            screen_width,
            screen_height,
            cursor_x,
            cursor_y,
            left: false,
            middle: false,
            right: false,
        };

        let screen_state = ScreenState::new(cursor_x, cursor_y, lfb);

        Ok(Self {
            keyboard_focus: Arc::new(Mutex::new(None)?),
            mouse_state: Arc::new(Mutex::new(mouse_state)?),
            screen_state: Arc::new(Mutex::new(screen_state)?),
        })
    }

    pub fn send_keypress(&self, code: char) -> Result<()> {
        let (server2client, portal_id) = if let Some(PortalRef {
            server2client,
            portal_id: id,
        }) = &*self.keyboard_focus.lock().unwrap()
        {
            (server2client.clone(), *id)
        } else {
            return Ok(());
        };

        let mut server2client = server2client.lock().unwrap();
        ipc::send_message(
            &mut *server2client,
            &Event::Input {
                portal_id,
                input: EventInput::KeyPress { code },
            },
        )
    }

    pub fn update_mouse_state(&self, dx: i16, dy: i16, dw: i8, left: bool, middle: bool, right: bool) -> Result<()> {
        let (x, y, inputs) = self.mouse_state.lock().unwrap().update(dx, dy, dw, left, middle, right);
        self.screen_state.lock().unwrap().update_mouse_state(x, y, inputs)
    }
}

pub struct ServerApp {
    portals_by_id: HashMap<usize, Entity>,
    input: ServerInput,
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl ServerApp {
    pub fn new() -> Result<Self> {
        let mut systems: Vec<Box<dyn System>> = Vec::new();
        let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        let lfb_mem = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };
        let lfb = FrameBuffer::from_os_mem(800.0, 600.0, lfb_mem);
        let input = ServerInput::new(lfb)?;
        systems.push(Box::new(ServerPortalSystem::new(
            input.screen_state.clone(),
            input.keyboard_focus.clone(),
        )));

        Ok(Self {
            portals_by_id: HashMap::new(),
            input,
            world: World::new(),
            systems,
        })
    }

    pub fn handle_command(
        &mut self,
        client_process: &Process,
        server2client: &Arc<Mutex<File>>,
        command: Command,
    ) -> Result<()> {
        match command {
            Command::Checkpoint { id } => {
                let mut server2client = server2client.lock().unwrap();
                ipc::send_message(&mut *server2client, &Event::Checkpoint { id })?;
            }

            Command::CreatePortal {
                id,
                pos,
                shared_mem_handle,
            } => {
                let shared_mem = SharedMem::from_raw(client_process.open_handle(shared_mem_handle)?, false);
                let portal = ServerPortal::new(&self.world, server2client.clone(), id, pos, shared_mem)?;
                let entity = self.world.spawn((portal,));
                self.portals_by_id.insert(id, entity);
            }

            Command::DestroyPortal { id } => {
                if let Some(entity) = self.portals_by_id.remove(&id) {
                    self.world.despawn(entity).unwrap();
                }
            }

            Command::InvalidatePortal { id } => {
                if let Some(entity) = self.portals_by_id.get(&id).copied() {
                    self.world.get_mut::<ServerPortal>(entity).unwrap().invalidate();
                }
            }

            Command::MovePortal { id, pos } => {
                if let Some(entity) = self.portals_by_id.get(&id).copied() {
                    self.world.get_mut::<ServerPortal>(entity).unwrap().move_to(pos);
                }
            }
        }

        for system in self.systems.iter_mut() {
            system.run(&mut self.world)?;
        }

        Ok(())
    }

    pub fn input(&self) -> ServerInput {
        self.input.clone()
    }
}
