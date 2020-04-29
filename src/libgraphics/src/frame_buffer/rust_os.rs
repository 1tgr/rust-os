use crate::Result;
use core::ops::{Deref, DerefMut};
use os::{OSHandle, SharedMem};

pub struct FrameBuffer {
    data: SharedMem,
}

impl FrameBuffer {
    pub fn from_raw(size: (u16, u16), handle: OSHandle) -> Result<Self> {
        let len = crate::frame_buffer::byte_len(size);
        let data = SharedMem::from_raw(handle, len, false)?;
        Ok(Self { data })
    }

    pub fn new(size: (u16, u16)) -> Result<Self> {
        let len = crate::frame_buffer::byte_len(size);
        let data = SharedMem::new(len, true)?;
        Ok(Self { data })
    }

    pub fn resize(&mut self, size: (u16, u16)) -> Result<()> {
        let len = crate::frame_buffer::byte_len(size);
        self.data.resize(len)?;
        Ok(())
    }

    pub fn as_raw(&self) -> usize {
        self.data.as_handle().get()
    }
}

impl Deref for FrameBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.data
    }
}

impl DerefMut for FrameBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}
