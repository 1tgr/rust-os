use crate::ipc;
use crate::server::portal::{PortalRef, ServerPortal, ServerPortalSystem};
use crate::types::{Command, Event};
use alloc::collections::btree_map::{BTreeMap, Entry};
use alloc::sync::Arc;
use cairo::bindings::*;
use ecs::{ComponentStorage, Entity};
use os::{File, Mutex, OSMem, Process, Result, SharedMem};

pub struct ServerInput {
    input_state: Arc<Mutex<Option<PortalRef>>>,
}

impl ServerInput {
    pub fn send_keypress(&self, code: char) -> Result<()> {
        let (server2client, portal_id) =
            if let Some(PortalRef { server2client, id }) = &*self.input_state.lock().unwrap() {
                (server2client.clone(), *id)
            } else {
                return Ok(());
            };

        let mut server2client = server2client.lock().unwrap();
        ipc::send_message(&mut *server2client, &Event::KeyPress { portal_id, code })
    }
}

pub struct ServerApp {
    portals_by_id: BTreeMap<usize, Entity>,
    input_state: Arc<Mutex<Option<PortalRef>>>,
    entities: ComponentStorage,
}

impl ServerApp {
    pub fn new() -> Result<Self> {
        let mut entities = ComponentStorage::new();
        let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
        let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
        let lfb_mem = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };
        let input_state = Arc::new(Mutex::new(None)?);
        entities.add_system(ServerPortalSystem::new(lfb_mem, input_state.clone()));

        Ok(Self {
            portals_by_id: BTreeMap::new(),
            input_state,
            entities,
        })
    }

    pub fn handle_command(
        &mut self,
        client_process: &Process,
        server2client: &Arc<Mutex<File>>,
        command: Command,
    ) -> Result<()> {
        println!("[Server] {:?}", command);
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
                let portal = Entity::new();
                let shared_mem = SharedMem::from_raw(client_process.open_handle(shared_mem_handle)?, false);
                self.entities.add_component(
                    &portal,
                    ServerPortal::new(&self.entities, server2client.clone(), id, pos, shared_mem)?,
                );

                self.portals_by_id.insert(id, portal);
            }

            Command::DestroyPortal { id } => {
                if let Entry::Occupied(entry) = self.portals_by_id.entry(id) {
                    self.entities.clear_entity(entry.get());
                    entry.remove();
                }
            }

            Command::InvalidatePortal { id } => {
                if let Some(portal) = self.portals_by_id.get(&id) {
                    self.entities.update_component(portal, |state: &mut ServerPortal| {
                        state.invalidate();
                        Ok(())
                    })?;
                }
            }

            Command::MovePortal { id, pos } => {
                if let Some(portal) = self.portals_by_id.get(&id) {
                    self.entities.update_component(portal, |state: &mut ServerPortal| {
                        state.move_to(pos);
                        Ok(())
                    })?;
                }
            }
        }

        self.entities.run_systems()
    }

    pub fn input(&self) -> ServerInput {
        ServerInput {
            input_state: self.input_state.clone(),
        }
    }
}
