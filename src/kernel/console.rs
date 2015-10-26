use alloc::arc::Arc;
use arch::keyboard::keys;
use core::char;
use io::{AsyncRead,FlatMap,Read,Write};
use prelude::*;
use process::KObj;

pub struct Console {
    input: FlatMap,
    output: Arc<KObj>
}

impl Console {
    pub fn new(input: Arc<KObj>, output: Arc<KObj>) -> Self {
        Console {
            output: output.clone(),
            input: FlatMap::new(input, 4, move |keys| {
                let c = unsafe { *(keys[0..4].as_ptr() as *const u32) };
                let keys = keys::Bucky::from_bits_truncate(c);
                let c = c & !keys.bits();
                if keys.intersects(keys::BUCKY_RELEASE | keys::BUCKY_CTRL | keys::BUCKY_ALT | keys::BUCKY_ALTGR) {
                    Vec::new()
                } else if let Some(c) = char::from_u32(c) {
                    let mut bytes = vec![0; 4];
                    let len = char::encode_utf8(c, &mut bytes).unwrap();
                    bytes.truncate(len);

                    if let Some(output) = output.write() {
                        let _ = output.write(&bytes[..]);
                    }

                    bytes
                } else {
                    Vec::new()
                }
            }, |left| {
                left.iter().position(|b| *b == 10).map(|index| {
                    let mut right = left.split_off(index);
                    right.remove(0);
                    right
                })
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
        self.output.write()
    }
}
