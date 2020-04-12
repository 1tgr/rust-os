use crate::arch::cpu;
use crate::arch::isr::{self, DropIrqHandler};
use crate::io::{AsyncRead, Pipe, Write};
use crate::kobj::KObj;
use crate::spin::Mutex;
use alloc::sync::Arc;
use core::char;

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

//                  S    C    C+S  AGr  AGr+S
pub struct Key(u32, u32, u32, u32, u32, u32);

pub mod keys {
    // "bucky bits"
    bitflags! {
        pub struct Bucky: u32 {
            const BUCKY_RELEASE =   0x01000000; // Key was released
            const BUCKY_ALT =       0x02000000; // Alt is pressed
            const BUCKY_CTRL =      0x04000000; // Ctrl is pressed
            const BUCKY_SHIFT =     0x08000000; // Shift is pressed
            const BUCKY_CAPS =      0x10000000; // CapsLock is on
            const BUCKY_NUM =       0x20000000; // NumLock is on
            const BUCKY_SCRL =      0x40000000; // ScrollLock is on
            const BUCKY_ALTGR =     0x80000000;  // AltGr is pressed
        }
    }

    // "ASCII" values for non-ASCII keys. All of these are user-defined.
    // function keys:
    pub const F1: u32 = 0xE000;
    pub const F2: u32 = F1 + 1;
    pub const F3: u32 = F2 + 1;
    pub const F4: u32 = F3 + 1;
    pub const F5: u32 = F4 + 1;
    pub const F6: u32 = F5 + 1;
    pub const F7: u32 = F6 + 1;
    pub const F8: u32 = F7 + 1;
    pub const F9: u32 = F8 + 1;
    pub const F10: u32 = F9 + 1;
    pub const F11: u32 = F10 + 1;
    pub const F12: u32 = F11 + 1; // 0x10B
                                  // cursor keys
    pub const INS: u32 = F12 + 1; // 0x10C
    pub const DEL: u32 = INS + 1;
    pub const HOME: u32 = DEL + 1;
    pub const END: u32 = HOME + 1;
    pub const PGUP: u32 = END + 1;
    pub const PGDN: u32 = PGUP + 1;
    pub const LEFT: u32 = PGDN + 1;
    pub const UP: u32 = LEFT + 1;
    pub const DOWN: u32 = UP + 1;
    pub const RIGHT: u32 = DOWN + 1; // 0x115
                                     // print screen/sys rq and pause/break
    pub const PRTSC: u32 = RIGHT + 1; // 0x116
    pub const PAUSE: u32 = PRTSC + 1; // 0x117
                                      // these return a value but they could also act as additional bucky keys
    pub const LWIN: u32 = PAUSE + 1; // 0x118
    pub const RWIN: u32 = LWIN + 1;
    pub const MENU: u32 = RWIN + 1; // 0x11A

    pub const SYSR: u32 = Bucky::BUCKY_ALT.bits | PRTSC;
}

impl Key {
    pub fn pick(&self, keys: keys::Bucky) -> u32 {
        if keys.contains(keys::Bucky::BUCKY_SHIFT | keys::Bucky::BUCKY_ALTGR) {
            self.5
        } else if keys.contains(keys::Bucky::BUCKY_SHIFT | keys::Bucky::BUCKY_CTRL) {
            self.3
        } else if keys.contains(keys::Bucky::BUCKY_ALTGR) {
            self.4
        } else if keys.contains(keys::Bucky::BUCKY_CTRL) {
            self.2
        } else if keys.contains(keys::Bucky::BUCKY_SHIFT) {
            self.1
        } else {
            self.0
        }
    }
}

struct KeyboardState {
    extended: bool,
    keys: keys::Bucky,
    compose: u32,
}

enum Keypress {
    Char(u32),
    Scancode(keys::Bucky, u8, bool),
    Leds(u8),
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            extended: false,
            keys: keys::Bucky::empty(),
            compose: 0,
        }
    }

    fn decode(&mut self, code: u8) -> Option<Keypress> {
        const RAW_CTRL: u8 = 0x1D;
        const RAW_LEFT_SHIFT: u8 = 0x2A;
        const RAW_CAPS_LOCK: u8 = 0x3A;
        const RAW_ALT: u8 = 0x38;
        const RAW_RIGHT_SHIFT: u8 = 0x36;
        const RAW_SCROLL_LOCK: u8 = 0x46;
        const RAW_NUM_LOCK: u8 = 0x45;
        const RAW_NUM7: u8 = 0x47;
        const RAW_NUM0: u8 = 0x52;

        let down = (code & 0x80) == 0;
        let code = code & !0x80;

        if code == 0x60 {
            self.extended = true;
            return None;
        }

        let extended = self.extended;
        if extended {
            self.extended = false;
        }

        match code {
            RAW_CTRL => {
                self.keys.set(keys::Bucky::BUCKY_CTRL, down);
                None
            }

            RAW_ALT => {
                if extended {
                    self.keys.set(keys::Bucky::BUCKY_ALTGR, down);
                    None
                } else {
                    self.keys.set(keys::Bucky::BUCKY_ALT, down);

                    if down && self.keys.contains(keys::Bucky::BUCKY_ALT) {
                        self.compose = 0;
                        None
                    } else if !down && self.keys.contains(keys::Bucky::BUCKY_ALT) {
                        if self.compose != 0 {
                            Some(Keypress::Char(self.compose))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }

            RAW_LEFT_SHIFT | RAW_RIGHT_SHIFT => {
                self.keys.set(keys::Bucky::BUCKY_SHIFT, down);
                None
            }

            RAW_NUM_LOCK | RAW_CAPS_LOCK | RAW_SCROLL_LOCK => {
                if down {
                    let k = match code {
                        RAW_NUM_LOCK => keys::Bucky::BUCKY_NUM,
                        RAW_CAPS_LOCK => keys::Bucky::BUCKY_CAPS,
                        _ => keys::Bucky::BUCKY_SCRL,
                    };

                    self.keys.toggle(k);

                    let mut flags = 0;
                    if self.keys.contains(keys::Bucky::BUCKY_SCRL) {
                        flags |= 1;
                    }

                    if self.keys.contains(keys::Bucky::BUCKY_NUM) {
                        flags |= 2;
                    }

                    if self.keys.contains(keys::Bucky::BUCKY_CAPS) {
                        flags |= 4;
                    }

                    Some(Keypress::Leds(flags))
                } else {
                    None
                }
            }

            _ => {
                let num = if code >= RAW_NUM7 {
                    static KEYPAD: &'static [Option<u8>; (RAW_NUM0 - RAW_NUM7 + 1) as usize] = &[
                        Some(7),
                        Some(8),
                        Some(9),
                        None,
                        Some(4),
                        Some(5),
                        Some(6),
                        None,
                        Some(1),
                        Some(2),
                        Some(3),
                        Some(0),
                    ];

                    match KEYPAD.get((code - RAW_NUM7) as usize) {
                        Some(&Some(num)) => Some(num),
                        _ => None,
                    }
                } else {
                    None
                };

                match num {
                    Some(num) if self.keys.contains(keys::Bucky::BUCKY_ALT) => {
                        if down {
                            self.compose = self.compose * 10 + num as u32;
                        }
                        None
                    }

                    Some(num) if self.keys.contains(keys::Bucky::BUCKY_NUM) => {
                        Some(Keypress::Scancode(self.keys, '0' as u8 + num, down))
                    }

                    _ => Some(Keypress::Scancode(self.keys, code, down)),
                }
            }
        }
    }

    pub fn translate(&mut self, code: u8) -> Option<[u8; 4]> {
        let c = match self.decode(code)? {
            Keypress::Char(c) => c,

            Keypress::Scancode(mut keys, scan, down) => {
                keys.set(keys::Bucky::BUCKY_RELEASE, !down);

                let key = british::KEYS.get(scan as usize)?;
                let c = key.pick(keys);

                let c = match char::from_u32(c) {
                    Some(c) if keys.contains(keys::Bucky::BUCKY_CAPS) => c.to_uppercase().next().unwrap_or(c) as u32,
                    _ => c,
                };

                keys.bits() | c
            }

            Keypress::Leds(flags) => {
                unsafe {
                    cpu::outb(0x60, 0xed);
                    cpu::outb(0x60, flags);
                }
                return None;
            }
        };

        Some(c.to_le_bytes())
    }
}

mod british;

pub struct Keyboard {
    _drop_irq_handler: DropIrqHandler,
    device: Arc<Pipe>,
}

impl Keyboard {
    pub fn new() -> Self {
        let device = Arc::new(Pipe::new());

        let handler = {
            let state = Mutex::new(KeyboardState::new());
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
            _drop_irq_handler: isr::register_irq_handler(1, handler),
            device,
        }
    }
}

impl KObj for Keyboard {
    fn async_read(&self) -> Option<&dyn AsyncRead> {
        Some(&*self.device)
    }
}
