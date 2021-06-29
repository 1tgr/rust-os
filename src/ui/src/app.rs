use crate::geometry::ScreenPoint;
use crate::input::InputDb;
use crate::pipe::ClientPipe;
use crate::property_map::PropertyMap;
use crate::render::RenderDb;
use crate::widget::WidgetId;
use alloc::rc::Rc;
use alloc::vec::Vec;
use euclid::Rect;
use hashbrown::HashMap;
use os::SharedMem;
use raqote::DrawTarget;
use ui_types::types::{Checkpoint, Command, CreatePortal, DrawPortal, Event, EventInput, ScreenSpace};
use ui_types::Result;

pub trait IntoPropertyMap {
    fn into_property_map(self, map: &mut PropertyMap) -> WidgetId;
}

pub trait ExtendPropertyMap {
    fn extend_property_map(&self, parent_id: WidgetId, map: &mut PropertyMap);
}

impl<T> ExtendPropertyMap for Vec<T>
where
    T: ExtendPropertyMap,
{
    fn extend_property_map(&self, parent_id: WidgetId, map: &mut PropertyMap) {
        for item in self {
            item.extend_property_map(parent_id, map)
        }
    }
}

pub fn run<Db, F, I>(mut db: Db, mut render: F) -> Result<()>
where
    Db: InputDb + RenderDb,
    F: FnMut(&Db) -> I,
    I: IntoPropertyMap,
{
    let mut pipe = ClientPipe::new();
    let mut state = None;
    let mut busy_shared_mems = HashMap::new();
    let mut idle_shared_mems: Vec<SharedMem<u32>> = Vec::new();
    let mut next_frame_buffer_id = 1;
    loop {
        let into_property_map = render(&db);
        let mut parent_id = None;
        db.update_properties(|properties| {
            let properties_mut = Rc::make_mut(properties);
            properties_mut.clear();
            parent_id = Some(into_property_map.into_property_map(properties_mut));
        });

        let parent_id = parent_id.unwrap();

        let new_state = db.render_state(parent_id);
        if state.as_ref().map_or(true, |state| !new_state.eq(state)) {
            let pos: Rect<f32, ScreenSpace> = Rect::new(
                db.origin(parent_id).unwrap_or_default().cast_unit(),
                db.size(parent_id).unwrap_or_default().cast_unit(),
            );

            let int_pos = pos.round_out().to_i32();
            let size = int_pos.size;
            let len = size.width as usize * size.height as usize;

            let mut shared_mem = if let Some(mut shared_mem) = idle_shared_mems.pop() {
                shared_mem.resize(len)?;
                shared_mem
            } else {
                SharedMem::new(len, true)?
            };

            {
                let mut target = DrawTarget::from_backing(size.width, size.height, &mut *shared_mem);
                new_state.render_to(&mut target);
            }

            let frame_buffer_id = next_frame_buffer_id;
            next_frame_buffer_id += 1;

            let shared_mem_handle = shared_mem.as_handle().get();
            busy_shared_mems.insert(frame_buffer_id, shared_mem);

            if state.is_none() {
                pipe.send_command(&Command::CreatePortal(CreatePortal {
                    id: 0,
                    pos,
                    size,
                    frame_buffer_id,
                    shared_mem_handle,
                }))?;

                pipe.send_command(&Command::Checkpoint(Checkpoint { id: 0 }))?;
            } else {
                pipe.send_command(&Command::DrawPortal(DrawPortal {
                    id: 0,
                    size,
                    frame_buffer_id,
                    shared_mem_handle,
                }))?;
            }

            state = Some(new_state);
        }

        match pipe.wait_for_event()? {
            Event::Checkpoint { id: 0 } => {
                println!("System ready");
            }

            Event::Input { portal_id, input } if portal_id == 0 => match input {
                EventInput::MouseMove { info } => db.set_mouse_pos(ScreenPoint::new(info.x, info.y)),
                EventInput::KeyPress { code } => todo!("KeyPress({})", code),
                EventInput::MouseButtonDown { button, info } => {
                    db.set_mouse_down_at(button, Some(ScreenPoint::new(info.x, info.y)))
                }
                EventInput::MouseButtonUp { button, info: _ } => db.set_mouse_down_at(button, None),
            },

            Event::ReuseFrameBuffer { frame_buffer_id } => {
                if let Some(shared_mem) = busy_shared_mems.remove(&frame_buffer_id) {
                    idle_shared_mems.push(shared_mem);
                }
            }

            _ => {}
        }
    }
}
