use alloc::arc::Arc;
use arch::keyboard::{keys,Keyboard};
use core::char;
use io::{AsyncRead,Pipe,Read,Write};
use prelude::*;
use process::KObj;

pub struct Console {
    input: Pipe<Keyboard, Vec<u8>>,
    output: Arc<Write>
}

impl Console {
    pub fn new(input: Arc<Keyboard>, output: Arc<Write>) -> Self {
        Console {
            output: output.clone(),
            input: Pipe::new(input, 4, move |keys| {
                let c = unsafe { *(keys[0..4].as_ptr() as *const u32) };
                let keys = keys::Bucky::from_bits_truncate(c);
                let c = c & !keys.bits();
                if keys.intersects(keys::BUCKY_RELEASE | keys::BUCKY_CTRL | keys::BUCKY_ALT | keys::BUCKY_ALTGR) {
                    Vec::new()
                } else if let Some(c) = char::from_u32(c) {
                    let mut bytes = vec![0; 4];
                    let len = char::encode_utf8(c, &mut bytes).unwrap();
                    bytes.truncate(len);

                    let _ = output.write(&bytes[..]);
                    bytes
                } else {
                    Vec::new()
                }
            }, |left| {
                left.iter().position(|b| *b == 10).map(|index| left.split_off(index))
            })
        }
    }
}

impl KObj for Console {
    fn async_read(&self) -> Option<&AsyncRead> {
        Some(&self.input)
    }

    fn read(&self) -> Option<&Read> {
        Some(&self.input)
    }

    fn write(&self) -> Option<&Write> {
        Some(&*self.output)
    }
}
