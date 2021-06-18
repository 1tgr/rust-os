use crate::{OSHandle, Result};
use alloc::boxed::Box;
use syscall;
use syscall::ErrNum;

#[lang = "termination"]
pub trait Termination {
    fn report(self) -> i32;
}

impl Termination for () {
    fn report(self) -> i32 {
        Ok(()).report()
    }
}

impl Termination for Result<()> {
    fn report(self) -> i32 {
        self.map(|()| 0).report()
    }
}

impl Termination for i32 {
    fn report(self) -> i32 {
        -self
    }
}

impl Termination for ErrNum {
    fn report(self) -> i32 {
        (self as i32).report()
    }
}

impl Termination for Result<i32> {
    fn report(self) -> i32 {
        self.unwrap_or_else(|num| num.report())
    }
}

extern "C" fn thread_entry<T>(context: usize)
where
    T: Termination,
{
    let b: Box<Box<dyn FnOnce() -> T>> = unsafe { Box::from_raw(context as *mut _) };
    let code = b().report();
    Thread::exit(code)
}

pub struct Thread(OSHandle);

impl Thread {
    pub fn current_thread_id() -> usize {
        #[thread_local]
        static mut CURRENT_THREAD_ID: usize = 0;

        unsafe {
            if CURRENT_THREAD_ID == 0 {
                CURRENT_THREAD_ID = syscall::current_thread_id();
            }

            CURRENT_THREAD_ID
        }
    }

    pub fn from_raw(handle: OSHandle) -> Self {
        Thread(handle)
    }

    fn spawn_inner<T>(b: Box<Box<dyn FnOnce() -> T>>) -> Self
    where
        T: Termination,
    {
        let context_ptr = Box::into_raw(b);
        let handle = syscall::spawn_thread(thread_entry::<T>, context_ptr as usize);
        Self(OSHandle::from_raw(handle))
    }

    pub fn spawn<F, T>(entry: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
        T: Termination,
    {
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
