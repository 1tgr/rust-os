use ops::Deref;
use os::{OSHandle,OSMem,Result};
use syscall;

fn align_down(value: usize, round: usize) -> usize {
    value & !(round - 1)
}

fn align_up(value: usize, round: usize) -> usize {
    align_down(value + round - 1, round)
}

pub struct SharedMem {
    handle: OSHandle,
    writable: bool,
    len: usize,
    ptr: Option<OSMem>
}

impl SharedMem {
    pub fn create(writable: bool) -> Result<Self> {
        let handle = OSHandle::from_raw(try!(syscall::create_shared_mem()));
        Ok(SharedMem::open(handle, writable))
    }

    pub fn open(handle: OSHandle, writable: bool) -> Self {
        SharedMem {
            handle: handle,
            writable: writable,
            len: 0,
            ptr: None
        }
    }

    pub fn handle(&self) -> &OSHandle {
        &self.handle
    }

    pub fn resize(&mut self, new_len: usize) -> Result<()> {
        if align_up(new_len, 4096) == align_up(self.len, 4096) {
            return Ok(());
        }

        self.len = new_len;
        if new_len == 0 {
            self.ptr = None;
        } else {
            self.ptr = Some(OSMem::from_raw(try!(syscall::map_shared_mem(*self.handle, self.len, self.writable))));
        }

        Ok(())
    }
}

impl Deref for SharedMem {
    type Target = OSMem;

    fn deref(&self) -> &OSMem {
        self.ptr.as_ref().unwrap()
    }
}
