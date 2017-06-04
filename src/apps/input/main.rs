#![feature(link_args)]
#![feature(start)]

extern crate os;
extern crate syscall;

use os::Process;
use syscall::{ErrNum,Result};
use syscall::libc_helpers::{stdin,stdout};

fn read_line() -> Result<String> {
    let mut v = Vec::new();
    loop {
        let mut buf = vec![0; 100];
        let bytes = syscall::read(unsafe { stdin }, &mut buf[..])?;
        if bytes < buf.len() {
            buf.truncate(bytes);
            v.extend(buf);
            break;
        }

        v.extend(buf);
    }

    String::from_utf8(v).map_err(|_| ErrNum::Utf8Error)
}

#[cfg_attr(target_arch = "x86_64", link_args = "-T ../../libsyscall/arch/amd64/link.ld")]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    let inherit = unsafe { [ stdin, stdout ] };
    loop {
        print!("> ");

        match read_line() {
            Ok(line) => {
                if line == "exit" {
                    return 0;
                } else if line.len() > 0 {
                    Process::spawn(&line, &inherit).unwrap().wait_for_exit().unwrap();
                }
            },

            Err(code) => { return -(code as isize) }
        }
    }
}
