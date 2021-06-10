use crate::pipe::ServerPipe;
use crate::property::{PixelSize, SharedMemHandle};
use crate::screen::{Screen, ScreenState, MOUSE_BUTTONS};
use crate::server_portal::ServerPortal;
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::mem;
use euclid::{Size2D, Vector2D};
use os::{File, Thread};
use raqote::SolidSource;
use std::io::Read;
use sw_composite::blend::Src;
use ui::app::IntoPropertyMap;
use ui::input::InputDb;
use ui::render::{RenderDb, RenderStateObject};
use ui_macros::render;
use ui_types::Result;

fn mouse_thread(screen: Arc<Screen>) -> Result<()> {
    let mut mouse = File::open("ps2_mouse")?;
    loop {
        let mut buf = [0; 6];
        mouse.read_exact(&mut buf)?;

        #[derive(Debug)]
        struct MouseEvent {
            dx: i16,
            dy: i16,
            dw: i8,
            buttons: u8,
        }

        let MouseEvent { dx, dy, dw, buttons } = unsafe { mem::transmute::<[u8; 6], MouseEvent>(buf) };
        let delta = Vector2D::new(dx, dy).to_i32();
        let buttons = [(buttons & 4) != 0, (buttons & 2) != 0, (buttons & 1) != 0];
        screen.update(move |state| state.update_mouse_state_delta(delta, dw, buttons));
    }
}

fn client_thread(screen: Arc<Screen>) -> Result<()> {
    ServerPipe::spawn("ui-demo")?.run(|process, server2client, command| {
        screen.try_update(move |state| state.handle_command(process, server2client, command))
    })
}

pub fn run<Db>(mut db: Db) -> Result<()>
where
    Db: InputDb + RenderDb,
{
    let screen_size = Size2D::new(800, 600);
    let screen = ScreenState::init(screen_size)?;
    let screen = Arc::new(Screen::new(screen));

    Thread::spawn({
        let screen = screen.clone();
        move || mouse_thread(screen).unwrap()
    });

    Thread::spawn({
        let screen = screen.clone();
        move || client_thread(screen).unwrap()
    });

    let mut prev_state: Option<Rc<dyn RenderStateObject>> = None;

    loop {
        let (mouse_pos, mouse_down, portals) = {
            let screen = screen.wait_for_update();
            (
                screen.mouse_pointer.pos.to_f32(),
                screen.mouse_down,
                screen.portals.clone(),
            )
        };

        db.set_mouse_pos(mouse_pos);

        for (&button, &down) in MOUSE_BUTTONS.iter().zip(mouse_down.iter()) {
            if !down {
                db.set_mouse_down_at(button, None);
            } else if db.mouse_down_at(button).is_none() {
                db.set_mouse_down_at(button, Some(mouse_pos));
            }
        }

        let into_property_map = render! {
            <panel
                size={screen_size.to_f32().cast_unit()}
                color={SolidSource::from_unpremultiplied_argb(0xff, 0, 0, 0)}
            >
                for portal in portals {
                    <server_portal
                        key={portal.id.to_string()}
                        origin={portal.pos.origin.cast_unit()}
                        pixel_size={portal.size}
                        shared_mem_handle={portal.shared_mem_handle.clone()}
                    />
                }
            </panel>
        };

        let mut parent_id = None;
        db.update_properties(|properties| {
            let properties_mut = Rc::make_mut(properties);
            properties_mut.clear();
            parent_id = Some(into_property_map.into_property_map(properties_mut));
        });

        let state = db.render_state(parent_id.unwrap());
        if prev_state.as_ref().map_or(true, |prev_state| !prev_state.eq(&state)) {
            let bounds = state.bounds();
            let bounds = prev_state
                .map_or(bounds, |prev_state| prev_state.bounds().union(&bounds))
                .round_out()
                .to_i32();

            let ScreenState {
                changed: _,
                lfb_back,
                lfb,
                mouse_pointer: mouse,
                mouse_down: _,
                portals: _,
            } = &mut *screen.state.lock();

            let mut target = lfb_back.as_draw_target_mut();
            target.push_clip_rect(bounds.to_untyped());
            state.render_to(&mut target);
            target.pop_clip();

            if mouse.pointer_rect().intersects(&bounds) {
                mouse.draw(lfb_back);
            }

            lfb.draw_sprite_region_at(lfb_back, bounds, bounds.min, Src);
            prev_state = Some(state);
        }
    }
}
