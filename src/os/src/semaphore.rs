use crate::OSHandle;

pub struct Semaphore(OSHandle);

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(value: usize) -> Self {
        Self(OSHandle::from_raw(syscall::create_semaphore(value)))
    }
}

impl Semaphore {
    pub fn wait(&self) {
        syscall::wait_semaphore(self.0.get()).unwrap();
    }

    pub fn post(&self) {
        syscall::post_semaphore(self.0.get()).unwrap()
    }
}
