use crate::portal::{PortalRef, ServerPortal};
use alloc::sync::Arc;
use graphics_base::frame_buffer::FrameBuffer;
use graphics_base::ipc;
use graphics_base::system::System;
use graphics_base::types::{Command, Event};
use graphics_base::Result;
use hashbrown::HashMap;
use hecs::{Entity, World};
use os::{File, Mutex, Process};

pub struct ServerApp {
    portals_by_id: HashMap<usize, Entity>,
    world: World,
    systems: Vec<Box<dyn System>>,
}

impl ServerApp {
    pub fn new() -> Self {
        Self {
            portals_by_id: HashMap::new(),
            world: World::new(),
            systems: Vec::new(),
        }
    }

    pub fn add_system<S>(&mut self, system: S)
    where
        S: System + 'static,
    {
        self.systems.push(Box::new(system));
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
                size: frame_buffer_size,
                frame_buffer_id,
                shared_mem_handle,
            } => {
                let portal_ref = PortalRef {
                    portal_id: id,
                    server2client: server2client.clone(),
                };

                let shared_mem_handle = client_process.open_handle(shared_mem_handle)?;
                let frame_buffer = FrameBuffer::from_raw(frame_buffer_size, shared_mem_handle)?;

                let portal = ServerPortal::new(
                    &self.world,
                    portal_ref,
                    pos,
                    frame_buffer_id,
                    frame_buffer_size,
                    frame_buffer,
                );

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
                size: frame_buffer_size,
                frame_buffer_id,
                shared_mem_handle,
            } => {
                if let Some(entity) = self.portals_by_id.get(&id).copied() {
                    let shared_mem_handle = client_process.open_handle(shared_mem_handle)?;
                    let frame_buffer = FrameBuffer::from_raw(frame_buffer_size, shared_mem_handle)?;

                    let frame_buffer_id = self.world.get_mut::<ServerPortal>(entity).unwrap().draw(
                        frame_buffer_id,
                        frame_buffer_size,
                        frame_buffer,
                    );

                    let mut server2client = server2client.lock().unwrap();
                    ipc::send_message(&mut *server2client, &Event::ReuseFrameBuffer { frame_buffer_id })?;
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
}
