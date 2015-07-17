use ::arch::cpu;
use ::phys_mem;
use core::intrinsics;
use libc::c_char;
use spin::Mutex;

struct VgaState {
    base_ptr: *mut u16,
    ptr: *mut u16,
    row: isize,
    col: isize
}

impl VgaState {
    pub fn new() -> VgaState {
        let base_ptr = unsafe { phys_mem::phys2virt::<u16>(0xb8000) as *mut u16 };

        let state = VgaState {
            base_ptr: base_ptr,
            ptr: base_ptr,
            row: 0,
            col: 0
        };

        let mut ptr = base_ptr;
        unsafe {
            for _ in 0..80 * 25 {
                *ptr = 0x1700;
                ptr = ptr.offset(1);
            }
        }

        state.update_cursor();
        state
    }

    fn update_cursor(&self) {
        let position = self.row * 80 + self.col;
        let low = position & 0xff;
        let high = (position >> 8) & 0xff;
     
        unsafe {
            cpu::outb(0x3D4, 0x0F);
            cpu::outb(0x3D5, low as u8);
            cpu::outb(0x3D4, 0x0E);
            cpu::outb(0x3D5, high as u8);
        }
    }

    fn newline(&mut self) {
        let col = self.col;
        self.col = 0;

        if self.row < 24 {
            self.row += 1;
            self.ptr = unsafe { self.ptr.offset(80 - col) };
        } else {
            unsafe {
                intrinsics::copy(self.base_ptr.offset(80), self.base_ptr, 80 * 24);

                self.ptr = self.base_ptr.offset(80 * 24);

                let mut ptr = self.ptr;
                for _ in 0..80 {
                    *ptr = 0x1700;
                    ptr = ptr.offset(1);
                }
            }
        }
    }

    pub fn putb(&mut self, b: u8) {
        match b {
            10 => self.newline(),
            _ => {
                unsafe { *self.ptr = 0x1700 | (b as u16); }

                self.col += 1;

                if self.col >= 80 {
                    self.newline();
                } else {
                    self.ptr = unsafe { self.ptr.offset(1) };
                }
            }
        }
    }
}

lazy_static! {
    static noalloc STATE: Mutex<VgaState> = Mutex::new(VgaState::new());
}

pub fn puts(s: &str) {
    let mut state = lock!(STATE);
    for b in s.bytes() {
        state.putb(b);
    }

    state.update_cursor();
}

pub unsafe fn put_cstr(s: *const c_char) {
    let mut state = lock!(STATE);
    let mut s = s;
    while *s != 0 {
        state.putb(*s as u8);
        s = s.offset(1);
    }

    state.update_cursor();
}
