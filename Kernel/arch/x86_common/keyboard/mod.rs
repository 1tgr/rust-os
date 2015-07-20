use ::arch::cpu;
use ::arch::debug;
use ::arch::isr::{self,DropIrqHandler};
use ::phys_mem::PhysicalBitmap;
use ::process::Process;
use ::thread::{Deferred,Promise,Scheduler};
use ::virt_mem::VirtualTree;
use spin::Mutex;
use std::char;
use std::cmp;
use std::collections::VecDeque;
use std::mem;
use std::slice::bytes;
use std::sync::Arc;

//                  S    C    C+S  AGr  AGr+S
pub struct Key(u32, u32, u32, u32, u32, u32);

pub mod keys {
    // "bucky bits"
    bitflags! {
        flags Bucky: u32 {
            const BUCKY_RELEASE =   0x01000000, // Key was released
            const BUCKY_ALT =       0x02000000, // Alt is pressed
            const BUCKY_CTRL =      0x04000000, // Ctrl is pressed
            const BUCKY_SHIFT =     0x08000000, // Shift is pressed
            const BUCKY_CAPS =      0x10000000, // CapsLock is on
            const BUCKY_NUM =       0x20000000, // NumLock is on
            const BUCKY_SCRL =      0x40000000, // ScrollLock is on
            const BUCKY_ALTGR =     0x80000000  // AltGr is pressed
        }
    }

    impl Bucky {
        pub fn set(&mut self, other: Bucky, set: bool) {
            if set {
                self.insert(other)
            } else {
                self.remove(other)
            }
        }
    }

    // "ASCII" values for non-ASCII keys. All of these are user-defined.
    // function keys:
    pub const F1: u32 =      0xE000;
    pub const F2: u32 =      F1 + 1;
    pub const F3: u32 =      F2 + 1;
    pub const F4: u32 =      F3 + 1;
    pub const F5: u32 =      F4 + 1;
    pub const F6: u32 =      F5 + 1;
    pub const F7: u32 =      F6 + 1;
    pub const F8: u32 =      F7 + 1;
    pub const F9: u32 =      F8 + 1;
    pub const F10: u32 =     F9 + 1;
    pub const F11: u32 =     F10 + 1;
    pub const F12: u32 =     F11 + 1;  // 0x10B
    // cursor keys
    pub const INS: u32 =     F12 + 1;  // 0x10C
    pub const DEL: u32 =     INS + 1;
    pub const HOME: u32 =    DEL + 1;
    pub const END: u32 =     HOME + 1;
    pub const PGUP: u32 =    END + 1;
    pub const PGDN: u32 =    PGUP + 1;
    pub const LEFT: u32 =    PGDN + 1;
    pub const UP: u32 =      LEFT + 1;
    pub const DOWN: u32 =    UP + 1;
    pub const RIGHT: u32 =   DOWN + 1;  // 0x115
    // print screen/sys rq and pause/break
    pub const PRTSC: u32 =   RIGHT + 1; // 0x116
    pub const PAUSE: u32 =   PRTSC + 1; // 0x117
    // these return a value but they could also act as additional bucky keys
    pub const LWIN: u32 =    PAUSE + 1; // 0x118
    pub const RWIN: u32 =    LWIN + 1;
    pub const MENU: u32 =    RWIN + 1;  // 0x11A

    pub const SYSR: u32 =    BUCKY_ALT.bits | PRTSC;
}

impl Key {
    pub fn pick(&self, keys: keys::Bucky) -> u32 {
        if keys.contains(keys::BUCKY_SHIFT | keys::BUCKY_ALTGR) {
            self.5
        } else if keys.contains(keys::BUCKY_SHIFT | keys::BUCKY_CTRL) {
            self.3
        } else if keys.contains(keys::BUCKY_ALTGR) {
            self.4
        } else if keys.contains(keys::BUCKY_CTRL) {
            self.2
        } else if keys.contains(keys::BUCKY_SHIFT) {
            self.1
        } else {
            self.0
        }
    }
}

const RAW_LEFT_CTRL: u16 =   0x1D;
const RAW_LEFT_SHIFT: u16 =  0x2A;
//const RAW_CAPS_LOCK: u16 =   0x3A;
const RAW_LEFT_ALT: u16 =    0x38;
const RAW_RIGHT_ALT: u16 =   0x6038;
const RAW_RIGHT_CTRL: u16 =  0x601D;
const RAW_RIGHT_SHIFT: u16 = 0x36;
//const RAW_SCROLL_LOCK: u16 = 0x46;
//const RAW_NUM_LOCK: u16 =    0x45;
const RAW_NUM7: u16 =        0x47;
//const RAW_NUM8: u16 =        0x48;
//const RAW_NUM9: u16 =        0x49;
//const RAW_NUM4: u16 =        0x4b;
//const RAW_NUM5: u16 =        0x4c;
//const RAW_NUM6: u16 =        0x4d;
//const RAW_NUM1: u16 =        0x4f;
//const RAW_NUM2: u16 =        0x50;
//const RAW_NUM3: u16 =        0x51;
const RAW_NUM0: u16 =        0x52;

struct KeyboardState {
    extended: bool,
    keys: keys::Bucky,
    compose: u32,
}

impl KeyboardState {
    pub fn new() -> KeyboardState {
        KeyboardState {
            extended: false,
            keys: keys::Bucky::empty(),
            compose: 0
        }
    }

    pub fn decode(&mut self, scancode: u8) -> u32 {
        static KEYPAD: &'static [Option<u8>; (RAW_NUM0 - RAW_NUM7 + 1) as usize] = &[
            Some(7), Some(8), Some(9), None,
            Some(4), Some(5), Some(6), None,
            Some(1), Some(2), Some(3),
            Some(0)
        ];

        let down = (scancode & 0x80) == 0;
        let mut code = (scancode & !0x80) as u16;
        let mut key: u32 = 0;

        if code == 0x60 {
            self.extended = true;
            return 0;
        } else if self.extended {
            code |= 0x6000;
            self.extended = false;
        }

        match code {
            RAW_LEFT_CTRL | RAW_RIGHT_CTRL => {
                self.keys.set(keys::BUCKY_CTRL, down);
            },

            RAW_LEFT_ALT => {
                if down && self.keys.contains(keys::BUCKY_ALT) {
                    self.compose = 0;
                } else if !down && self.keys.contains(keys::BUCKY_ALT) {
                    key =
                        if self.compose != 0 {
                            self.compose
                        } else {
                            (keys::BUCKY_ALT | keys::BUCKY_RELEASE).bits()
                        };
                }

                self.keys.set(keys::BUCKY_ALT, down);
            },

            RAW_RIGHT_ALT => {
                self.keys.set(keys::BUCKY_ALTGR, down);
            },

            RAW_LEFT_SHIFT | RAW_RIGHT_SHIFT => {
                self.keys.set(keys::BUCKY_SHIFT, down);
            },

            _ => {
                code &= !0x6000;
                if code >= RAW_NUM7 && code <= RAW_NUM0 && code != 0x4a && code != 0x4e {
                    match KEYPAD[(code - RAW_NUM7) as usize] {
                        Some(n) => {
                            if self.keys.contains(keys::BUCKY_ALT) {
                                if down {
                                    self.compose *= 10;
                                    self.compose += n as u32;
                                }
                            } else if self.keys.contains(keys::BUCKY_NUM) {
                                key = '0' as u32 + n as u32;
                            }
                        },

                        None => {
                            key = british::KEYS[code as usize].pick(keys::Bucky::empty());
                        }
                    }
                } else {
                    key = british::KEYS[code as usize].pick(self.keys);
                }

                /* if self.keys.contains(keys::BUCKY_CAPS) {
                    if (iswupper(key)) {
                        key = towlower(key);
                    } else if (iswlower(key)) {
                        key = towupper(key);
                    }
                } */
                
                if !down {
                    key |= keys::BUCKY_RELEASE.bits();
                }
            }
        }
        
        key | self.keys.bits()
    }
}

mod british;

pub trait Device {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>>;
}

pub struct IoRequest {
    buf: Vec<u8>,
    d: Deferred<Result<(Vec<u8>, usize), &'static str>>,
    current: usize
}

impl IoRequest {
    pub fn new(buf: Vec<u8>, d: Deferred<Result<(Vec<u8>, usize), &'static str>>) -> IoRequest {
        IoRequest {
            buf: buf,
            d: d,
            current: 0
        }
    }

    pub fn fulfil(mut self, data: &mut &[u8]) -> Option<IoRequest> {
        {
            let len = cmp::min(self.buf.len(), data.len());
            let (data1, data2) = data.split_at(len);
            *data = data2;

            let buf1 = &mut self.buf[self.current .. self.current + len];
            bytes::copy_memory(data1, buf1);
            self.current += len;
        }

        if self.current >= self.buf.len() {
            self.d.resolve(Ok((self.buf, self.current)));
            None
        } else {
            Some(self)
        }
    }
}

pub struct Keyboard {
    _drop_irq_handler: DropIrqHandler,
    scheduler: Arc<Scheduler>,
    requests: Arc<Mutex<VecDeque<IoRequest>>>
}

impl Keyboard {
    pub fn new(scheduler: Arc<Scheduler>) -> Keyboard {
        let requests = Arc::new(Mutex::new(VecDeque::new()));
        let state = Mutex::new(KeyboardState::new());

        let handler = {
            let requests = requests.clone();
            move || {
                let data: [u8; 4] = {
                    let scancode = unsafe { cpu::inb(0x60) };
                    let key = lock!(state).decode(scancode);
                    unsafe { mem::transmute(key) }
                };

                let mut slice: &[u8] = &data;
                loop {
                    if slice.len() == 0 {
                        break;
                    }

                    let request: IoRequest =
                        match lock!(requests).pop_front() {
                            Some(request) => request,
                            None => { break; }
                        };

                    if let Some(request) = request.fulfil(&mut slice) {
                        lock!(requests).push_front(request);
                    }
                }
            }
        };

        Keyboard {
            _drop_irq_handler: isr::register_irq_handler(1, handler),
            scheduler: scheduler,
            requests: requests
        }
    }
}

impl<'a> Device for Keyboard {
    fn read_async(&self, buf: Vec<u8>) -> Deferred<Result<(Vec<u8>, usize), &'static str>> {
        let d = Deferred::new(self.scheduler.clone());
        lock!(self.requests).push_back(IoRequest::new(buf, d.clone()));
        d
    }
}

test! {
    fn can_read_key() {
        let phys = Arc::new(PhysicalBitmap::parse_multiboot());
        let kernel_virt = Arc::new(VirtualTree::new());
        let p = Arc::new(Process::new(phys, kernel_virt).unwrap());
        let scheduler = Arc::new(Scheduler::new(p));
        let keyboard = Keyboard::new(scheduler);
        log!("Type something and press Enter");

        loop {
            let d = keyboard.read_async(vec![0; 4]);
            let c: u32;
            loop {
                if let Some(result) = d.try_get() {
                    let (keys, _) = result.unwrap(); 
                    let p = keys[0..4].as_ptr() as *const u32;
                    c = unsafe { *p };
                    break;
                } else {
                    unsafe { asm!("hlt") };
                }
            }

            if let Some(c) = char::from_u32(c & !keys::BUCKY_SHIFT.bits()) {
                if c == '\n' {
                    break;
                } else if c != '\0' {
                    let mut s = String::new();
                    s.push(c);
                    debug::puts(&s);
                }
            }
        }
    }
}
