extern crate alloc_system;
extern crate rt;

use os::Process;
use syscall::libc_helpers::{stdin, stdout};
use syscall::{ErrNum, Result};

fn read_line(buf: &mut Vec<u8>) -> Result<String> {
    loop {
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let line = buf.drain(..pos + 1).collect();
            return String::from_utf8(line).map_err(|_| ErrNum::Utf8Error);
        }

        let index = buf.len();
        buf.resize(index + 100, 0);

        let len = syscall::read(stdin, &mut buf[index..])?;
        buf.truncate(index + len);
    }
}

fn main() -> Result<()> {
    let inherit = [stdin, stdout];
    let mut buf = Vec::new();
    loop {
        print!("> ");

        let mut line = read_line(&mut buf)?;
        assert_eq!(line.pop(), Some('\n'));
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
