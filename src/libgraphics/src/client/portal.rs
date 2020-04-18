use crate::client;
use crate::client::pipe::ClientPipe;
use crate::components::{NeedsPaint, OnPaint, Position};
use crate::frame_buffer::FrameBuffer;
use crate::system::{ChangedIndex, DeletedIndex, System};
use crate::types::Command;
use crate::Result;
use alloc::rc::Rc;
use core::cell::RefCell;
use hecs::World;
use os::SharedMem;

#[derive(Copy, Clone)]
pub struct ClientPortalId(pub usize);

pub struct ClientPortal;

pub struct ClientPortalSystem {
    pipe: Rc<RefCell<ClientPipe>>,
    deleted_index: DeletedIndex<ClientPortalId>,
    prev_position_index: ChangedIndex<Position>,
}

impl ClientPortalSystem {
    pub fn new(pipe: Rc<RefCell<ClientPipe>>) -> Self {
        Self {
            pipe,
            deleted_index: DeletedIndex::new(),
            prev_position_index: ChangedIndex::new(),
        }
    }
}

impl System for ClientPortalSystem {
    fn run(&mut self, world: &mut World) -> Result<()> {
        let mut pipe = self.pipe.borrow_mut();

        let new_portals = world
            .query::<&Position>()
            .with::<ClientPortal>()
            .without::<ClientPortalId>()
            .iter()
            .map(|(entity, &Position(pos))| (entity, pos))
            .collect::<Vec<_>>();

        for (entity, pos) in new_portals {
            let id = client::alloc_id();
            let shared_mem = SharedMem::new(true)?;

            pipe.send_command(&Command::CreatePortal {
                id,
                pos,
                shared_mem_handle: shared_mem.handle().get(),
            })?;

            world
                .insert(
                    entity,
                    (
                        ClientPortalId(id),
                        NeedsPaint,
                        FrameBuffer::from_shared_mem(pos.width, pos.height, shared_mem)?,
                    ),
                )
                .unwrap();
        }

        let changed_position = self
            .prev_position_index
            .update(world.query::<&Position>().with::<ClientPortalId>().iter());

        for &entity in changed_position.keys() {
            if let Some(q) = world.query_one::<(&ClientPortalId, &Position)>(entity) {
                let (&ClientPortalId(id), &Position(pos)) = q.get();
                pipe.send_command(&Command::MovePortal { id, pos })?;
            }
        }

        let mut painted_entities = Vec::new();
        for (entity, (&ClientPortalId(id), frame_buffer, on_paint)) in world
            .query::<(&ClientPortalId, &mut FrameBuffer, &OnPaint)>()
            .with::<NeedsPaint>()
            .iter()
        {
            let OnPaint(on_paint) = on_paint;
            let cr = frame_buffer.as_surface().into_cairo();
            cr.set_source_rgb(1.0, 1.0, 1.5);
            cr.paint();
            (on_paint)(world, entity, &cr);
            pipe.send_command(&Command::InvalidatePortal { id })?;
            painted_entities.push(entity);
        }

        for entity in painted_entities {
            world.remove_one::<NeedsPaint>(entity).unwrap();
        }

        for (_, ClientPortalId(id)) in self.deleted_index.update(world.query::<&ClientPortalId>().iter()) {
            pipe.send_command(&Command::DestroyPortal { id })?;
        }

        Ok(())
    }
}
