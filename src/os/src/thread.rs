use super::{OSHandle, Result};
use alloc::boxed::Box;
use syscall;

extern "C" fn thread_entry(context: usize) {
    let b: Box<Box<dyn FnOnce() -> i32>> = unsafe { Box::from_raw(context as *mut _) };
    let code = b();
    Thread::exit(code)
}

pub struct Thread(OSHandle);

impl Thread {
    pub fn from_raw(handle: OSHandle) -> Self {
        Thread(handle)
    }

    fn spawn_inner(b: Box<Box<dyn FnOnce() -> i32>>) -> Self {
        let context_ptr = Box::into_raw(b);
        let handle = syscall::spawn_thread(thread_entry, context_ptr as usize);
        Self(OSHandle::from_raw(handle))
    }

    pub fn spawn<T: 'static + FnOnce() -> i32 + Send>(entry: T) -> Self {
        Self::spawn_inner(Box::new(Box::new(entry)))
    }

    pub fn exit(code: i32) -> ! {
        syscall::exit_thread(code)
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }

    pub fn wait_for_exit(&self) -> Result<()> {
        syscall::wait_for_exit(self.0.get())?;
        Ok(())
    }
}
