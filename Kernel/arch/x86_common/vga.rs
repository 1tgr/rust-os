use ::phys_mem;
use core::intrinsics;
use libc::c_char;
use spin::{Mutex,MutexGuard};
use super::io;

struct VgaState {
    base_ptr: *mut u16,
    row: isize,
    col: isize
}

unsafe fn update_cursor(state: &VgaState) {
    let position = state.row * 80 + state.col;
    let low = position & 0xff;
    let high = (position >> 8) & 0xff;
 
    io::outb(0x3D4, 0x0F);
    io::outb(0x3D5, low as u8);
    io::outb(0x3D4, 0x0E);
    io::outb(0x3D5, high as u8);
}

unsafe fn newline<'a>(state: &mut MutexGuard<'a, VgaState>, ptr: *mut u16) -> *mut u16 {
    let col = state.col;
    state.col = 0;

    if state.row < 24 {
        state.row += 1;
        ptr.offset(80 - col)
    } else {
        intrinsics::copy(state.base_ptr.offset(80), state.base_ptr, 80 * 24);

        let line_base_ptr = state.base_ptr.offset(80 * 24);
        let mut ptr = line_base_ptr;
        for _ in 0..80 {
            *ptr = 0x1700;
            ptr = ptr.offset(1);
        }

        line_base_ptr
    }
}

unsafe fn putb<'a>(state: &mut MutexGuard<'a, VgaState>, ptr: *mut u16, b: u8) -> *mut u16 {
    return
        match b {
            10 => newline(state, ptr),

            _ => {
                *ptr = 0x1700 | (b as u16);
                state.col += 1;

                if state.col >= 80 {
                    newline(state, ptr)
                } else {
                    ptr.offset(1)
                }
            }
        }
}

lazy_static! {
    static ref STATE: Mutex<VgaState> = {
        let state = VgaState {
            base_ptr: phys_mem::phys2virt::<u16>(0xb8000) as *mut u16,
            row: 0,
            col: 0
        };

        let mut ptr = state.base_ptr;
        for _ in 0..80 * 25 {
            *ptr = 0x1700;
            ptr = ptr.offset(1);
        }

        update_cursor(&state);
        Mutex::new(state)
    };
}

pub fn puts(s: &str) {
    let mut state = STATE.lock();
    unsafe {
        let mut ptr = state.base_ptr.offset(state.row * 80 + state.col);
        for b in s.bytes() {
            ptr = putb(&mut state, ptr, b);
        }

        update_cursor(&mut state);
    }
}

pub unsafe fn put_cstr(s: *const c_char) {
    let mut state = STATE.lock();
    let mut ptr = state.base_ptr.offset(state.row * 80 + state.col);
    let mut s = s;
    while *s != 0 {
        ptr = putb(&mut state, ptr, *s as u8);
        s = s.offset(1);
    }

    update_cursor(&mut state);
}
