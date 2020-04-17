use crate::arch::cpu;
use crate::arch::isr::{self, DropIrqHandler};
use crate::io::{AsyncRead, Pipe, Read, Write};
use crate::kobj::KObj;
use crate::spin::Mutex;
use alloc::sync::Arc;
use core::mem;

unsafe fn write_keyboard(port: u16, data: u8) {
    loop {
        let stat = cpu::inb(0x64);

        if (stat & 0x02) == 0 {
            break;
        }
    }

    cpu::outb(port, data);
}

unsafe fn read_keyboard() -> u8 {
    loop {
        let stat = cpu::inb(0x64);

        if (stat & 0x01) != 0 {
            let data = cpu::inb(0x60);

            if (stat & 0xc0) == 0 {
                return data;
            }
        }
    }
}

unsafe fn write_ps2_aux(data: u8) {
    write_keyboard(0x64, 0xd4);
    write_keyboard(0x60, data);

    let response = read_keyboard();
    assert_eq!(response, 0xfa);
}

struct Ps2MouseState {
    has_wheel: bool,
    data: [u8; 4],
    len: usize,
}

impl Ps2MouseState {
    fn new(has_wheel: bool) -> Self {
        Self {
            has_wheel,
            data: [0; 4],
            len: 0,
        }
    }

    fn translate(&mut self, code: u8) -> Option<[u8; 6]> {
        self.data[self.len] = code;
        self.len += 1;

        if (self.len < 3 && !self.has_wheel) || (self.len < 4 && self.has_wheel) {
            return None;
        }

        let data = mem::replace(&mut self.data, [0; 4]);
        self.len = 0;

        let buttons = (data[0] & 0x04) >> 1 | /* middle */
            (data[0] & 0x02) >> 1 | /* right */
            (data[0] & 0x01) << 2; /* left */

        let dx = if (data[0] & 0x10) != 0 {
            data[1] as i16 - 256
        } else {
            data[1] as i16
        };

        let dy = if (data[0] & 0x20) != 0 {
            -(data[2] as i16 - 256)
        } else {
            -(data[2] as i16)
        };

        let dw = data[3] as i8;

        #[allow(dead_code)]
        struct MouseEvent {
            dx: i16,
            dy: i16,
            dw: i8,
            buttons: u8,
        }

        let response = MouseEvent { dx, dy, dw, buttons };
        Some(unsafe { mem::transmute::<MouseEvent, [u8; 6]>(response) })
    }
}

pub struct Ps2Mouse {
    _drop_irq_handler: DropIrqHandler,
    device: Arc<Pipe>,
}

impl Ps2Mouse {
    pub fn new() -> Self {
        let has_wheel = unsafe {
            write_keyboard(0x64, 0x20); // read controller RAM (controller command byte)
            let status = read_keyboard() & 0xfc; // clear interrupt bits
            write_keyboard(0x64, 0x60); // write controller RAM (controller command byte)
            write_keyboard(0x60, status);

            write_keyboard(0x64, 0xa8); // enable aux device

            for &data in &[
                0xf6, // reset
                0xf3, 200, // set sample rate = 200
                0xf3, 0x64, // set sample rate = 100
                0xf3, 0x50, // set sample rate = 80
            ] {
                write_ps2_aux(data);
            }

            write_ps2_aux(0xf2); // get device id

            let id = read_keyboard();

            for &data in &[
                0xf3, 0xff, // set sample rate = 255
            ] {
                write_ps2_aux(data);
            }

            write_ps2_aux(0xf4); // enable data reporting

            write_keyboard(0x64, 0x20); // read controller RAM (controller command byte)
            let status = read_keyboard() | 3; // set interrupt bits
            write_keyboard(0x64, 0x60); // write controller RAM (controller command byte)
            write_keyboard(0x60, status);

            id == 3
        };

        let device = Arc::new(Pipe::new());

        let handler = {
            let state = Mutex::new(Ps2MouseState::new(has_wheel));
            let device = device.clone();
            move || {
                let mut state = lock!(state);
                let code = unsafe { read_keyboard() };
                if let Some(bytes) = state.translate(code) {
                    let _ = Write::write(&*device, &bytes[..]);
                }
            }
        };

        Self {
            _drop_irq_handler: isr::register_irq_handler(12, handler),
            device,
        }
    }
}

impl KObj for Ps2Mouse {
    fn async_read(&self) -> Option<&dyn AsyncRead> {
        Some(&*self.device)
    }

    fn read(&self) -> Option<&dyn Read> {
        Some(&*self.device)
    }
}
