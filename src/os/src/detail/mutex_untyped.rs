use crate::OSHandle;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct UntypedMutex {
    counter: AtomicUsize,
    handle: OSHandle,
}

unsafe impl Send for UntypedMutex {}
unsafe impl Sync for UntypedMutex {}

impl UntypedMutex {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            handle: OSHandle::from_raw(syscall::create_mutex()),
        }
    }
}

impl UntypedMutex {
    pub fn lock(&self) {
        if self.counter.fetch_add(1, Ordering::Acquire) > 0 {
            syscall::lock_mutex(self.handle.get()).unwrap();
        }
    }

    pub fn unlock(&self) {
        if self.counter.fetch_sub(1, Ordering::Release) > 1 {
            syscall::unlock_mutex(self.handle.get()).unwrap();
        }
    }
}
