use collections::vec_deque::VecDeque;
use corepack;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::io::{Read,Write};
use syscall::{ErrNum,Result};

struct FromFrontIter<'a, T: 'a>(&'a mut VecDeque<T>);

impl<'a, T: 'a> Iterator for FromFrontIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

pub fn read_message<T: DeserializeOwned>(buf: &mut VecDeque<u8>, file: &mut Read) -> Result<T> {
    let mut temp = vec![0; 4096];
    loop {
        match corepack::from_iter(FromFrontIter(buf)) {
            Ok(message) => { return Ok(message) },
            Err(corepack::error::Error::EndOfStream) => { },
            Err(e) => panic!("Unexpected corepack error: {}", e)
        }

        let bytes_read = file.read(&mut temp)?;
        buf.extend(&temp[..bytes_read]);
    }
}

pub fn send_message<T: Serialize>(file: &mut Write, message: T) -> Result<()> {
    let buf = corepack::to_bytes(message).or(Err(ErrNum::NotSupported))?;
    file.write_all(&buf)?;
    Ok(())
}
