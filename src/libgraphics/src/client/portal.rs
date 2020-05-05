use crate::client;
use crate::client::pipe::ClientPipe;
use crate::components::{CapturesMouseInput, Focus, NeedsPaint, OnClick, OnInput, OnPaint, Parent, Position, Text};
use crate::frame_buffer::{AsSurfaceMut, FrameBuffer};
use crate::system::{ChangedIndex, DeletedIndex, System};
use crate::types::{Command, Event, EventInput, Rect};
use crate::widgets::{Button, ClientPortal, Label};
use crate::Result;
use cairo::bindings::CAIRO_FORMAT_RGB24;
use cairo::cairo::Cairo;
use hashbrown::{HashMap, HashSet};
use hecs::{Entity, World};

struct Decoration;

fn find_needs_paint_children(world: &World, entity: Entity, entities: &mut HashSet<Entity>) -> bool {
    let mut any = false;
    for (child, (&Parent(parent), needs_paint)) in world.query::<(&Parent, Option<&NeedsPaint>)>().iter() {
        if parent != entity {
            continue;
        }

        if needs_paint.is_some() {
            entities.insert(child);
            any = true;
        }

        if find_needs_paint_children(world, child, entities) {
            any = true;
        }
    }

    if any {
        entities.insert(entity);
    }

    any
}

fn render_tree(world: &World, entity: Entity, on_paint: Option<&OnPaint>, cr: &Cairo) {
    if let Some(OnPaint(on_paint)) = on_paint {
        (on_paint)(world, entity, &cr);
    }

    for (child, (&Parent(parent), &Position(pos), on_paint)) in world
        .query::<(&Parent, &Position, Option<&OnPaint>)>()
        .without::<Decoration>()
        .iter()
    {
        if parent != entity {
            continue;
        }

        cr.save()
            .rectangle(pos.x, pos.y, pos.width, pos.height)
            .clip()
            .translate(pos.x, pos.y);

        render_tree(world, child, on_paint, cr);

        cr.restore();
    }
}

fn portal_to_child(world: &World, parent: Entity, pos: Rect, x: f64, y: f64) -> (f64, f64) {
    let x = x - pos.x;
    let y = y - pos.y;
    world
        .query_one::<(&Parent, &Position)>(parent)
        .unwrap()
        .get()
        .map(|(&Parent(parent), &Position(pos))| portal_to_child(world, parent, pos, x, y))
        .unwrap_or((x, y))
}

fn hit_test(world: &World, entity: Entity, x: f64, y: f64) -> Option<(Entity, Option<OnInput>, f64, f64)> {
    for (child, (on_input, &Parent(parent), &Position(pos))) in
        world.query::<(Option<&OnInput>, &Parent, &Position)>().iter()
    {
        if parent != entity {
            continue;
        }

        if pos.contains(x, y) {
            let x = x - pos.x;
            let y = y - pos.y;
            return Some(hit_test(world, child, x, y).unwrap_or_else(|| (child, on_input.cloned(), x, y)));
        }
    }

    None
}

fn find_input_entity(
    world: &World,
    portal_id: usize,
    input: EventInput,
) -> Option<(Entity, Option<OnInput>, EventInput)> {
    for (entity, (&ClientPortalId(id), &Focus(focus), on_input)) in
        world.query::<(&ClientPortalId, &Focus, Option<&OnInput>)>().iter()
    {
        if portal_id != id {
            continue;
        }

        let tuple = match input {
            EventInput::KeyPress { .. } => {
                if let Some(entity) = focus {
                    let mut query = world.query_one::<Option<&OnInput>>(entity).unwrap();
                    let on_input = query.get().unwrap();
                    (entity, on_input.cloned(), input)
                } else {
                    (entity, on_input.cloned(), input)
                }
            }

            EventInput::Mouse { x, y, input } => {
                let x = x - 2.0;
                let y = y - 22.0;
                if let Some((entity, (on_input, parent, &Position(pos)))) = world
                    .query::<(Option<&OnInput>, Option<&Parent>, &Position)>()
                    .with::<CapturesMouseInput>()
                    .iter()
                    .next()
                {
                    let (x, y) = if let Some(&Parent(parent)) = parent {
                        portal_to_child(world, parent, pos, x, y)
                    } else {
                        (x, y)
                    };

                    (entity, on_input.cloned(), EventInput::Mouse { x, y, input })
                } else if let Some((entity, on_input, x, y)) = hit_test(world, entity, x, y) {
                    (entity, on_input, EventInput::Mouse { x, y, input })
                } else {
                    (entity, on_input.cloned(), EventInput::Mouse { x, y, input })
                }
            }
        };

        return Some(tuple);
    }

    None
}

#[derive(Copy, Clone)]
struct ClientPortalId(pub usize);

pub struct ClientPortalSystemPre;

impl System for ClientPortalSystemPre {
    fn run(&mut self, world: &mut World) -> Result<()> {
        struct HasDecoration;

        let new_portals = world
            .query::<&Position>()
            .with::<ClientPortal>()
            .without::<HasDecoration>()
            .iter()
            .map(|(entity, &Position(pos))| (entity, pos))
            .collect::<Vec<_>>();

        for (entity, pos) in new_portals {
            world.spawn((
                Label,
                Decoration,
                Parent(entity),
                Position::new(0.0, -20.0, pos.width - 24.0, 20.0),
                Text::new("Hello"),
            ));

            world.spawn((
                Button,
                Decoration,
                Parent(entity),
                Position::new(pos.width - 22.0, -20.0, 18.0, 18.0),
                Text::new("X"),
                OnClick::new(move |world, _entity| {
                    world.despawn(entity).unwrap();
                    Ok(())
                }),
            ));

            world.insert_one(entity, HasDecoration).unwrap();
        }

        Ok(())
    }
}

pub struct ClientPortalSystem {
    pub pipe: ClientPipe,
    idle_frame_buffers: Vec<FrameBuffer>,
    busy_frame_buffers: HashMap<usize, FrameBuffer>,
    deleted_index: DeletedIndex<ClientPortalId>,
    prev_position_index: ChangedIndex<Position>,
}

impl ClientPortalSystem {
    pub fn new() -> Self {
        Self {
            pipe: ClientPipe::new(),
            idle_frame_buffers: Vec::new(),
            busy_frame_buffers: HashMap::new(),
            deleted_index: DeletedIndex::new(),
            prev_position_index: ChangedIndex::new(),
        }
    }

    fn render_portal(
        &mut self,
        world: &World,
        entity: Entity,
        pos: Rect,
        on_paint: Option<&OnPaint>,
    ) -> Result<((u16, u16), usize, usize)> {
        let Rect { width, height, .. } = pos;
        let size = ((width + 0.5) as u16, (height + 0.5) as u16);

        let mut frame_buffer = if let Some(mut frame_buffer) = self.idle_frame_buffers.pop() {
            frame_buffer.resize(size)?;
            frame_buffer
        } else {
            FrameBuffer::new(size)?
        };

        {
            let cr = frame_buffer.as_surface_mut(CAIRO_FORMAT_RGB24, size).into_cairo();
            cr.set_source_rgb(0.98, 0.64, 0.066).paint().translate(2.0, 22.0);

            for (child, (&Parent(parent), &Position(pos), on_paint)) in world
                .query::<(&Parent, &Position, Option<&OnPaint>)>()
                .with::<Decoration>()
                .iter()
            {
                if parent != entity {
                    continue;
                }

                cr.save()
                    .rectangle(pos.x, pos.y, pos.width, pos.height)
                    .clip()
                    .translate(pos.x, pos.y);

                render_tree(world, child, on_paint, &cr);

                cr.restore();
            }

            cr.rectangle(0.0, 0.0, width - 4.0, height - 24.0)
                .clip()
                .save()
                .set_source_rgb(0.95, 0.95, 1.0)
                .paint()
                .restore();

            render_tree(world, entity, on_paint, &cr);
        }

        let frame_buffer_id = client::alloc_id();
        let shared_mem_handle = frame_buffer.as_raw();
        self.busy_frame_buffers.insert(frame_buffer_id, frame_buffer);
        Ok((size, frame_buffer_id, shared_mem_handle))
    }

    pub fn dispatch_event(&mut self, world: &mut World, event: Event) -> Result<()> {
        match event {
            Event::Input { portal_id, input } => {
                if let Some(tuple) = find_input_entity(world, portal_id, input) {
                    if let (entity, Some(OnInput(on_input)), input) = tuple {
                        (on_input)(world, entity, input)?;
                    }
                }
            }

            Event::ReuseFrameBuffer { frame_buffer_id } => {
                let frame_buffer = self.busy_frame_buffers.remove(&frame_buffer_id).unwrap();
                if self.idle_frame_buffers.len() < 5 {
                    self.idle_frame_buffers.push(frame_buffer);
                }
            }

            _ => (),
        }

        Ok(())
    }
}

impl System for ClientPortalSystem {
    fn run(&mut self, world: &mut World) -> Result<()> {
        let new_portals = world
            .query::<(&Position, Option<&OnPaint>)>()
            .with::<ClientPortal>()
            .without::<ClientPortalId>()
            .iter()
            .map(|(entity, (&Position(pos), on_paint))| (entity, pos, on_paint.cloned()))
            .collect::<Vec<_>>();

        for (entity, pos, on_paint) in new_portals {
            let id = client::alloc_id();
            let (size, frame_buffer_id, shared_mem_handle) =
                self.render_portal(world, entity, pos, on_paint.as_ref())?;

            self.pipe.send_command(&Command::CreatePortal {
                id,
                pos,
                size,
                frame_buffer_id,
                shared_mem_handle,
            })?;

            world.insert(entity, (ClientPortalId(id), Focus(None))).unwrap();
        }

        let changed_position = self
            .prev_position_index
            .update(world.query::<&Position>().with::<ClientPortalId>().iter());

        for &entity in changed_position.keys() {
            if let Ok(mut q) = world.query_one::<(&ClientPortalId, &Position)>(entity) {
                if let Some((&ClientPortalId(id), &Position(pos))) = q.get() {
                    self.pipe.send_command(&Command::MovePortal { id, pos })?;
                }
            }
        }

        let mut needs_paint_entities = HashSet::new();
        for (entity, ()) in world.query::<()>().with::<ClientPortalId>().iter() {
            if find_needs_paint_children(world, entity, &mut needs_paint_entities) {
                needs_paint_entities.insert(entity);
            }
        }

        for &entity in needs_paint_entities.iter() {
            world.insert_one(entity, NeedsPaint).unwrap();
        }

        for (entity, (&ClientPortalId(id), &Position(pos), on_paint)) in world
            .query::<(&ClientPortalId, &Position, Option<&OnPaint>)>()
            .with::<NeedsPaint>()
            .iter()
        {
            let (size, frame_buffer_id, shared_mem_handle) = self.render_portal(world, entity, pos, on_paint)?;
            self.pipe.send_command(&Command::DrawPortal {
                id,
                size,
                frame_buffer_id,
                shared_mem_handle,
            })?;
            needs_paint_entities.insert(entity);
        }

        for entity in needs_paint_entities {
            world.remove_one::<NeedsPaint>(entity).unwrap();
        }

        for (_, ClientPortalId(id)) in self.deleted_index.update(world.query::<&ClientPortalId>().iter()) {
            self.pipe.send_command(&Command::DestroyPortal { id })?;
        }

        Ok(())
    }
}
