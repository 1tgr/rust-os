use crate::components::{CapturesMouseInput, Focus, NeedsPaint, OnClick, OnInput, OnPaint, Parent, Position, Text};
use crate::pipe::{self, ClientPipe};
use crate::widgets::{Button, ClientPortal, Label};
use alloc::vec::Vec;
use cairo::bindings::CAIRO_FORMAT_RGB24;
use cairo::cairo::Cairo;
use graphics_base::frame_buffer::{AsSurfaceMut, FrameBuffer};
use graphics_base::system::{ChangedIndex, DeletedIndex, System};
use graphics_base::types::{Command, Event, EventInput, MouseButton, MouseInputInfo, Rect};
use graphics_base::Result;
use hashbrown::{HashMap, HashSet};
use hecs::{Entity, World};

struct Decoration;

struct DragDropState {
    origin: (f64, f64),
    direction: Rect,
}

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

fn find_input_portal(world: &World, portal_id: usize) -> Option<(Entity, Focus, Option<OnInput>)> {
    for (entity, (&ClientPortalId(id), focus, on_input)) in
        world.query::<(&ClientPortalId, &Focus, Option<&OnInput>)>().iter()
    {
        if portal_id == id {
            return Some((entity, focus.clone(), on_input.cloned()));
        }
    }

    None
}

fn find_keyboard_input_entity(
    world: &World,
    entity: Entity,
    focus: Focus,
    on_input: Option<OnInput>,
) -> (Entity, Option<OnInput>) {
    if let Focus(Some(focus)) = focus {
        if let Ok(mut query) = world.query_one::<Option<&OnInput>>(focus) {
            let on_input = query.get().unwrap();
            (entity, on_input.cloned())
        } else {
            (entity, on_input)
        }
    } else {
        (entity, on_input)
    }
}

fn find_mouse_input_entity(
    world: &World,
    entity: Entity,
    on_input: Option<OnInput>,
    info: &mut MouseInputInfo,
) -> (Entity, Option<OnInput>) {
    let input_capture = world
        .query::<(Option<&OnInput>, Option<&Parent>, &Position)>()
        .with::<CapturesMouseInput>()
        .iter()
        .next()
        .map(|(entity, (on_input, parent, position))| (entity, (on_input.cloned(), parent.cloned(), position.clone())));

    info.x -= 2.0;
    info.y -= 22.0;

    if let Some((entity, (on_input, parent, Position(pos)))) = input_capture {
        if let Some(Parent(parent)) = parent {
            let (x, y) = portal_to_child(world, parent, pos, info.x, info.y);
            info.x = x;
            info.y = y;
        }

        (entity, on_input)
    } else if let Some((entity, on_input, x, y)) = hit_test(world, entity, info.x, info.y) {
        info.x = x;
        info.y = y;
        (entity, on_input)
    } else {
        (entity, on_input)
    }
}

fn find_input_entity(
    world: &mut World,
    portal_id: usize,
    mut input: EventInput,
) -> Option<(Entity, Option<OnInput>, EventInput)> {
    let (entity, focus, on_input) = find_input_portal(world, portal_id)?;

    let (entity, on_input) = match input {
        EventInput::KeyPress { .. } => find_keyboard_input_entity(world, entity, focus, on_input),
        EventInput::MouseButtonDown { ref mut info, .. } => find_mouse_input_entity(world, entity, on_input, info),
        EventInput::MouseButtonUp { ref mut info, .. } => find_mouse_input_entity(world, entity, on_input, info),
        EventInput::MouseMove { ref mut info } => find_mouse_input_entity(world, entity, on_input, info),
    };

    Some((entity, on_input, input))
}

fn handle_portal_input(init_direction: Rect, world: &mut World, portal_entity: Entity, input: &EventInput) {
    match input {
        EventInput::MouseButtonDown {
            info,
            button: MouseButton::Left,
        } => {
            world
                .insert_one(
                    portal_entity,
                    DragDropState {
                        origin: (info.screen_x, info.screen_y),
                        direction: init_direction,
                    },
                )
                .unwrap();
        }

        EventInput::MouseMove { info } => {
            if let Some((
                Position(ref mut pos),
                DragDropState {
                    origin: ref mut prev_origin,
                    direction,
                },
            )) = world
                .query_one::<(&mut Position, &mut DragDropState)>(portal_entity)
                .unwrap()
                .get()
            {
                let origin = (info.screen_x, info.screen_y);
                let delta = (origin.0 - prev_origin.0, origin.1 - prev_origin.1);
                pos.x += delta.0 * direction.x;
                pos.y += delta.1 * direction.y;
                pos.width += delta.0 * direction.width;
                pos.height += delta.1 * direction.height;
                *prev_origin = origin;
            }
        }

        EventInput::MouseButtonUp {
            button: MouseButton::Left,
            ..
        } => {
            let _ = world.remove_one::<DragDropState>(portal_entity);
        }

        _ => (),
    }
}

#[derive(Copy, Clone)]
struct ClientPortalId(pub usize);

pub struct ClientPortalSystemPre;

impl System for ClientPortalSystemPre {
    fn run(&mut self, world: &mut World) -> Result<()> {
        struct HasDecoration;

        let new_portals = world
            .query::<(&Position, &Text)>()
            .with::<ClientPortal>()
            .without::<HasDecoration>()
            .iter()
            .map(|(entity, (&Position(pos), text))| (entity, pos, text.clone()))
            .collect::<Vec<_>>();

        for (portal_entity, pos, text) in new_portals {
            world.spawn((
                Label,
                Decoration,
                Parent(portal_entity),
                Position::new(0.0, -20.0, pos.width - 24.0, 20.0),
                text,
                OnInput::new(move |world, _entity, input| {
                    handle_portal_input(
                        Rect {
                            x: 1.0,
                            y: 1.0,
                            width: 0.0,
                            height: 0.0,
                        },
                        world,
                        portal_entity,
                        &input,
                    );
                    Ok(())
                }),
            ));

            world.spawn((
                Button,
                Decoration,
                Parent(portal_entity),
                Position::new(pos.width - 22.0, -20.0, 18.0, 18.0),
                Text::new("X"),
                OnClick::new(move |world, _entity| {
                    world.despawn(portal_entity).unwrap();
                    Ok(())
                }),
            ));

            world.insert_one(portal_entity, HasDecoration).unwrap();
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
            cr.save()
                .set_source_rgb(0.98, 0.64, 0.066)
                .paint()
                .restore()
                .translate(2.0, 22.0);

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

        let frame_buffer_id = pipe::alloc_id();
        let shared_mem_handle = frame_buffer.as_raw();
        self.busy_frame_buffers.insert(frame_buffer_id, frame_buffer);
        Ok((size, frame_buffer_id, shared_mem_handle))
    }

    pub fn dispatch_event(&mut self, world: &mut World, event: Event) -> Result<()> {
        match event {
            Event::Input { portal_id, input } => {
                if let Some(tuple) = find_input_entity(world, portal_id, input) {
                    if let (entity, Some(OnInput(on_input)), input) = tuple {
                        let input_capture = world.query_one::<&CapturesMouseInput>(entity).unwrap().get().cloned();

                        match input {
                            EventInput::MouseButtonDown { button, .. } => {
                                if input_capture.is_none() {
                                    world.insert_one(entity, CapturesMouseInput { button }).unwrap();
                                }
                            }

                            EventInput::MouseButtonUp { button, .. } => {
                                if let Some(CapturesMouseInput { button: prev_button }) = input_capture {
                                    if prev_button == button {
                                        let _ = world.remove_one::<CapturesMouseInput>(entity);
                                    }
                                }
                            }

                            _ => (),
                        }

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
            let id = pipe::alloc_id();
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
