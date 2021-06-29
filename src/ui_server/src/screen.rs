use crate::mouse::MousePointer;
use alloc::sync::Arc;
use core::mem;
use euclid::{Rect, Size2D, Vector2D};
use os::{File, Mutex, MutexGuard, OSHandle, OSMem, Process, Semaphore};
use sprite::Sprite;
use ui_types::ipc::send_message;
use ui_types::types::{
    Checkpoint, Command, CreatePortal, DrawPortal, Event, EventInput, MouseButton, MouseInputInfo, ScreenSpace,
};
use ui_types::Result;

pub const MOUSE_BUTTONS: [MouseButton; 3] = [MouseButton::Left, MouseButton::Middle, MouseButton::Right];

#[derive(Clone)]
pub struct Portal {
    pub process: Arc<Process>,
    pub server2client: Arc<Mutex<File>>,
    pub id: usize,
    pub pos: Rect<f32, ScreenSpace>,
    pub size: Size2D<i32, ScreenSpace>,
    pub frame_buffer_id: usize,
    pub shared_mem_handle: Arc<OSHandle>,
}

pub struct ScreenState {
    pub changed: bool,
    pub lfb_back: Sprite<OSMem<u32>, ScreenSpace>,
    pub lfb: Sprite<OSMem<u32>, ScreenSpace>,
    pub mouse_pointer: MousePointer,
    pub mouse_down: [bool; 3],
    pub portals: Vec<Portal>,
}

impl ScreenState {
    pub fn init(size: Size2D<i32, ScreenSpace>) -> Result<Self> {
        let lfb = syscall::init_video_mode(size.width as u16, size.height as u16, 32)?;
        let lfb = unsafe { OSMem::from_raw(lfb as *mut u32, size.width as usize * size.height as usize) };
        let lfb_back = OSMem::new(size.width as usize * size.height as usize)?;
        let mut lfb = Sprite::from_backing(size, lfb);
        let lfb_back = Sprite::from_backing(size, lfb_back);
        let mut mouse_pointer = MousePointer::init(size)?;
        mouse_pointer.draw(&mut lfb);

        Ok(Self {
            changed: false,
            lfb_back,
            lfb,
            mouse_pointer,
            mouse_down: [false; 3],
            portals: Vec::new(),
        })
    }

    pub fn update_mouse_state_delta(&mut self, delta: Vector2D<i32, ScreenSpace>, _dw: i8, buttons: [bool; 3]) {
        self.mouse_pointer
            .update_delta(&mut self.lfb_back, &mut self.lfb, delta);

        let prev_buttons = mem::replace(&mut self.mouse_down, buttons);
        let mouse_pos = self.mouse_pointer.pos.to_f32();
        if let Some(portal) = self.portals.iter().rev().find(|portal| portal.pos.contains(mouse_pos)) {
            let mut server2client = portal.server2client.lock();
            let portal_mouse_pos = mouse_pos - portal.pos.origin;

            let info = MouseInputInfo {
                x: portal_mouse_pos.x,
                y: portal_mouse_pos.y,
                screen_x: mouse_pos.x,
                screen_y: mouse_pos.y,
            };

            let _ = send_message(
                &mut *server2client,
                &Event::Input {
                    portal_id: portal.id,
                    input: EventInput::MouseMove { info: info.clone() },
                },
            );

            for ((&button, &prev_down), &down) in MOUSE_BUTTONS.iter().zip(prev_buttons.iter()).zip(buttons.iter()) {
                if !prev_down && down {
                    let _ = send_message(
                        &mut *server2client,
                        &Event::Input {
                            portal_id: portal.id,
                            input: EventInput::MouseButtonDown {
                                info: info.clone(),
                                button,
                            },
                        },
                    );
                } else if prev_down && !down {
                    let _ = send_message(
                        &mut *server2client,
                        &Event::Input {
                            portal_id: portal.id,
                            input: EventInput::MouseButtonUp {
                                info: info.clone(),
                                button,
                            },
                        },
                    );
                };
            }
        }
    }

    fn handle_checkpoint(
        &mut self,
        _process: &Arc<Process>,
        server2client: &Arc<Mutex<File>>,
        command: Checkpoint,
    ) -> Result<()> {
        let Checkpoint { id } = command;
        send_message(&mut *server2client.lock(), &Event::Checkpoint { id })
    }

    fn handle_create_portal(
        &mut self,
        process: &Arc<Process>,
        server2client: &Arc<Mutex<File>>,
        command: CreatePortal,
    ) -> Result<()> {
        let CreatePortal {
            id,
            pos,
            size,
            frame_buffer_id,
            shared_mem_handle,
        } = command;

        let process = process.clone();
        let server2client = server2client.clone();
        let shared_mem_handle = process.open_handle(shared_mem_handle)?;

        let portal = Portal {
            process,
            server2client,
            id,
            pos,
            size,
            frame_buffer_id,
            shared_mem_handle: Arc::new(shared_mem_handle),
        };

        self.portals.push(portal);
        Ok(())
    }

    fn handle_draw_portal(
        &mut self,
        process: &Arc<Process>,
        server2client: &Arc<Mutex<File>>,
        command: DrawPortal,
    ) -> Result<()> {
        let DrawPortal {
            id,
            size,
            frame_buffer_id,
            shared_mem_handle,
        } = command;

        if let Some(portal) = self
            .portals
            .iter_mut()
            .find(|portal| portal.process.handle() == process.handle() && portal.id == id)
        {
            portal.shared_mem_handle = Arc::new(process.open_handle(shared_mem_handle)?);
            portal.size = size;

            let frame_buffer_id = mem::replace(&mut portal.frame_buffer_id, frame_buffer_id);
            send_message(&mut *server2client.lock(), &Event::ReuseFrameBuffer { frame_buffer_id })?;
        }

        Ok(())
    }

    pub fn handle_command(
        &mut self,
        process: &Arc<Process>,
        server2client: &Arc<Mutex<File>>,
        command: Command,
    ) -> Result<()> {
        match command {
            Command::Checkpoint(command) => self.handle_checkpoint(process, server2client, command),
            Command::CreatePortal(command) => self.handle_create_portal(process, server2client, command),
            Command::DrawPortal(command) => self.handle_draw_portal(process, server2client, command),
            _ => todo!("{:?}", command),
        }
    }
}

pub struct Screen {
    pub state: Mutex<ScreenState>,
    pub semaphore: Semaphore,
}

impl Screen {
    pub fn new(state: ScreenState) -> Self {
        Self {
            state: Mutex::new(state),
            semaphore: Semaphore::new(0),
        }
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut ScreenState),
    {
        let mut state = self.state.lock();
        f(&mut *state);

        if !mem::replace(&mut state.changed, true) {
            self.semaphore.post();
        }
    }

    pub fn try_update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut ScreenState) -> Result<()>,
    {
        let mut state = self.state.lock();
        f(&mut *state)?;

        if !mem::replace(&mut state.changed, true) {
            self.semaphore.post();
        }

        Ok(())
    }

    pub fn wait_for_update(&self) -> MutexGuard<ScreenState> {
        self.semaphore.wait();

        let mut state = self.state.lock();
        state.changed = false;
        state
    }
}
