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

fn run() -> Result<()> {
    let inherit = unsafe { [ stdin, stdout ] };
    loop {
        print!("> ");

        let line = read_line()?;
        if line == "exit" {
            return Ok(());
        }

        if line.len() > 0 {
            let run_line = move || -> Result<()> {
                Process::spawn(&line, &inherit)?.wait_for_exit()?;
                Ok(())
            };

            if let Some(num) = run_line().err() {
                println!("{:?}", num);
            }
        }
    }
}

#[cfg_attr(target_arch = "x86_64", link_args = "-T ../../libsyscall/arch/amd64/link.ld")]
extern {
}

#[start]
#[no_mangle]
pub fn start(_: isize, _: *const *const u8) -> isize {
    run().map(|()| 0).unwrap_or_else(|num| -(num as isize))
}
