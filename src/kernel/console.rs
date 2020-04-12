use crate::arch::keyboard::keys;
use crate::io::{AsyncRead, FlatMap, Read, Write};
use crate::kobj::{KObj, KObjRef};
use crate::prelude::*;
use core::char;

pub struct Console {
    input: FlatMap,
    output: KObjRef<dyn Write>,
}

impl Console {
    pub fn new(input: KObjRef<dyn AsyncRead>, output: KObjRef<dyn Write>) -> Self {
        Console {
            output: output.clone(),
            input: FlatMap::new(
                input,
                4,
                move |v| {
                    assert_eq!(v.len(), 4);

                    let mut keys = [0; 4];
                    keys.copy_from_slice(&v);

                    let c = u32::from_le_bytes(keys);
                    let keys = keys::Bucky::from_bits_truncate(c);
                    let c = c & !keys.bits();
                    if keys.intersects(
                        keys::Bucky::BUCKY_RELEASE
                            | keys::Bucky::BUCKY_CTRL
                            | keys::Bucky::BUCKY_ALT
                            | keys::Bucky::BUCKY_ALTGR,
                    ) {
                        Vec::new()
                    } else if let Some(c) = char::from_u32(c) {
                        let mut bytes = vec![0; 4];
                        let byte_count = char::encode_utf8(c, &mut bytes[..]).len();
                        bytes.truncate(byte_count);

                        let _ = output.write(&bytes[..]);
                        bytes
                    } else {
                        Vec::new()
                    }
                },
                |left| {
                    left.iter().position(|b| *b == 10).map(|index| {
                        let mut right = left.split_off(index);
                        right.remove(0);
                        right
                    })
                },
            ),
        }
    }
}

impl KObj for Console {
    fn async_read(&self) -> Option<&dyn AsyncRead> {
        Some(&self.input)
    }

    fn read(&self) -> Option<&dyn Read> {
        Some(&self.input)
    }

    fn write(&self) -> Option<&dyn Write> {
        Some(&*self.output)
    }
}
