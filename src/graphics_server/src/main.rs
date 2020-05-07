extern crate alloc;
extern crate alloc_system;
extern crate rt;

use alloc::sync::Arc;
use cairo::bindings::*;
use core::mem;
use core::str;
use graphics::frame_buffer::AsSurfaceMut;
use graphics::server::{PortalRef, Screen, ServerApp, ServerPipe, ServerPortalSystem};
use graphics::EventInput;
use os::{File, Mutex, OSMem, Result, Thread};
use std::io::Read;

fn keyboard_thread(keyboard_focus: Arc<Mutex<Option<PortalRef>>>) -> Result<()> {
    let mut stdin = File::open("stdin")?;
    let mut buf = [0; 4];
    loop {
        let len = stdin.read(&mut buf)?;
        if let Ok(s) = str::from_utf8(&buf[..len]) {
            if let Some(code) = s.chars().next() {
                if let Some(ref portal_ref) = *keyboard_focus.lock().unwrap() {
                    portal_ref.send_input(EventInput::KeyPress { code })?;
                }
            }
        }
    }
}

fn mouse_thread<S>(screen: Arc<Mutex<Screen<S>>>) -> Result<()>
where
    S: AsSurfaceMut,
{
    let mut mouse = File::open("ps2_mouse")?;
    let mut buf = [0; 6];
    loop {
        let len = mouse.read(&mut buf)?;
        assert_eq!(len, buf.len());

        #[derive(Debug)]
        struct MouseEvent {
            dx: i16,
            dy: i16,
            dw: i8,
            buttons: u8,
        }

        let event = unsafe { mem::transmute::<[u8; 6], MouseEvent>(buf) };

        let buttons = [
            (event.buttons & 4) != 0,
            (event.buttons & 2) != 0,
            (event.buttons & 1) != 0,
        ];

        screen
            .lock()
            .unwrap()
            .update_mouse_state_delta(event.dx, event.dy, event.dw, buttons)?;
    }
}

fn main() -> Result<()> {
    let lfb_ptr = syscall::init_video_mode(800, 600, 32)?;
    let stride = cairo::stride_for_width(CAIRO_FORMAT_ARGB32, 800);
    let lfb = unsafe { OSMem::from_raw(lfb_ptr, stride * 600) };
    let screen = Arc::new(Mutex::new(Screen::new((800, 600), lfb))?);
    let keyboard_focus = Arc::new(Mutex::new(None)?);
    let mut app = ServerApp::new();
    app.add_system(ServerPortalSystem::new(screen.clone(), keyboard_focus.clone()));

    Thread::spawn(move || mouse_thread(screen).map(|()| 0).unwrap_or_else(|num| -(num as i32)))?;

    Thread::spawn(move || {
        keyboard_thread(keyboard_focus)
            .map(|()| 0)
            .unwrap_or_else(|num| -(num as i32))
    })?;

    ServerPipe::new(app, "graphics_client")?.run()
}
