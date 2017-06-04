use alloc::boxed::{Box,FnBox};
use super::{OSHandle,Result};
use syscall;

extern fn thread_entry(context: usize) {
    let b: Box<Box<FnBox() -> i32>> = unsafe { Box::from_raw(context as *mut _) };
    let code = b();
    Thread::exit(code)
}

fn spawn_inner(b: Box<Box<FnBox() -> i32>>) -> Result<Thread> {
    let context_ptr = Box::into_raw(b);

    let handle =
        match syscall::spawn_thread(thread_entry, context_ptr as usize) {
            Ok(handle) => handle,
            Err(code) => {
                unsafe { Box::from_raw(context_ptr) };
                return Err(code)
            }
        };

    Ok(Thread(OSHandle::from_raw(handle)))
}

pub struct Thread(OSHandle);

impl Thread {
    pub fn from_raw(handle: OSHandle) -> Self {
        Thread(handle)
    }

    pub fn spawn<T: 'static + FnOnce() -> i32>(entry: T) -> Result<Self> {
        spawn_inner(Box::new(Box::new(entry)))
    }

    pub fn exit(code: i32) -> ! {
        let _ = syscall::exit_thread(code);
        unreachable!()
    }

    pub fn handle(&self) -> &OSHandle {
        &self.0
    }

    pub fn wait_for_exit(&self) -> Result<()> {
        syscall::wait_for_exit(self.0.get())?;
        Ok(())
    }
}
