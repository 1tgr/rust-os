use ::phys_mem;
use core::intrinsics;
use libc::c_char;
use spin::{StaticMutex,STATIC_MUTEX_INIT};
use super::io;

static MUTEX: StaticMutex = STATIC_MUTEX_INIT;
static mut X: isize = 0;
static mut Y: isize = 0;

unsafe fn update_cursor() {
    let position = Y * 80 + X;
    let low = position & 0xff;
    let high = (position >> 8) & 0xff;
 
    io::outb(0x3D4, 0x0F);
    io::outb(0x3D5, low as u8);
    io::outb(0x3D4, 0x0E);
    io::outb(0x3D5, high as u8);
}

unsafe fn newline() {
    X = 0;

    if Y < 24 {
        Y += 1;
    } else {
        let base_ptr = phys_mem::phys2virt::<u16>(0xb8000) as *mut u16;
        intrinsics::copy(base_ptr.offset(80), base_ptr, 80 * 24);

        let mut ptr = base_ptr.offset(80 * 24);
        for _ in 0..80 {
            *ptr = 0x1700;
            ptr = ptr.offset(1);
        }
    }
}

unsafe fn putb(ptr: *mut u16, b: u8) -> *mut u16 {
    return
        match b {
            10 => {
                newline();
                ptr.offset(80 - X)
            },

            _ => {
                *ptr = 0x1700 | (b as u16);
                X += 1;

                if X >= 80 {
                    newline();
                    ptr.offset(81 - X)
                } else {
                    ptr.offset(1)
                }
            }
        }
}

pub fn puts(s: &str) {
    let _ = MUTEX.lock();
    unsafe {
        let base_ptr = phys_mem::phys2virt::<u16>(0xb8000) as *mut u16;
        let mut ptr = base_ptr.offset(Y * 80 + X);
        for b in s.bytes() {
            ptr = putb(ptr, b);
        }

        update_cursor();
    }
}

pub unsafe fn put_cstr(s: *const c_char) {
    let _ = MUTEX.lock();
    let base_ptr = phys_mem::phys2virt::<u16>(0xb8000) as *mut u16;
    let mut ptr = base_ptr.offset(Y * 80 + X);
    let mut s = s;
    while *s != 0 {
        ptr = putb(ptr, *s as u8);
        s = s.offset(1);
    }

    update_cursor();
}

pub fn init() {
    unsafe {
        let mut ptr = phys_mem::phys2virt::<u16>(0xb8000) as *mut u16;
        for _ in 0..80 * 25 {
            *ptr = 0x1700;
            ptr = ptr.offset(1);
        }

        update_cursor();
    }
}
