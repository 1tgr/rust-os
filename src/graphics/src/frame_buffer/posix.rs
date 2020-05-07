use crate::Result;
use alloc::borrow::Cow;
use core::ops::{Deref, DerefMut};
use core::slice;

pub struct FrameBuffer {
    data: Cow<'static, [u8]>,
}

impl FrameBuffer {
    pub fn from_raw(size: (u16, u16), handle: usize) -> Result<Self> {
        let p: *mut u8 = handle as *mut u8;
        let len = crate::frame_buffer::byte_len(size);
        let data = Cow::Borrowed(unsafe { slice::from_raw_parts(p, len) });
        Ok(Self { data })
    }

    pub fn new(size: (u16, u16)) -> Result<Self> {
        let len = crate::frame_buffer::byte_len(size);
        let data = Cow::Owned(vec![0; len]);
        Ok(Self { data })
    }

    pub fn resize(&mut self, size: (u16, u16)) -> Result<()> {
        let len = crate::frame_buffer::byte_len(size);

        match self.data {
            Cow::Borrowed(_) => panic!(),
            Cow::Owned(ref mut v) => {
                v.resize(len, 0);
            }
        };

        Ok(())
    }

    pub fn as_raw(&self) -> usize {
        let p: *const u8 = match self.data {
            Cow::Borrowed(ref slice) => slice.as_ptr(),
            Cow::Owned(ref v) => v.as_ptr(),
        };

        p as usize
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
        match self.data {
            Cow::Borrowed(_) => panic!(),
            Cow::Owned(ref mut v) => &mut *v,
        }
    }
}
