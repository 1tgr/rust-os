#![crate_name = "graphics"]

#[macro_use] extern crate serde_derive;

extern crate cairo;
extern crate corepack;
extern crate serde;
extern crate syscall;

mod types;

pub use types::*;

use std::io::{Read,Write};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use syscall::{ErrNum,Result};

pub fn read_message<T: DeserializeOwned>(file: &mut Read) -> Result<T> {
    let mut buf = Vec::new();
    loop {
        let offset = buf.len();
        buf.resize(offset + 16, 0);

        let bytes_read = file.read(&mut buf)?;
        buf.truncate(offset + bytes_read);
        match corepack::from_bytes(&buf) {
            Ok(message) => { return Ok(message) },
            Err(corepack::error::Error::EndOfStream) => { },
            Err(_) => { return Err(ErrNum::NotSupported) }
        }
    }
}

pub fn send_message<T: Serialize>(file: &mut Write, message: T) -> Result<()> {
    let buf = corepack::to_bytes(message).or(Err(ErrNum::NotSupported))?;
    file.write_all(&buf)?;
    Ok(())
}
