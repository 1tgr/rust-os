use collections::vec_deque::VecDeque;
use kobj::KObj;
use spin;
use syscall::{ErrNum,Result};
use thread::{self,BlockedThread};

struct UntypedMutexState {
    waiters: VecDeque<BlockedThread>,
    locked: bool,
}

pub struct UntypedMutex {
    state: spin::Mutex<UntypedMutexState>,
}

unsafe impl Send for UntypedMutex { }
unsafe impl Sync for UntypedMutex { }

impl UntypedMutex {
    pub fn new() -> Self {
        UntypedMutex {
            state: spin::Mutex::new(UntypedMutexState {
                waiters: VecDeque::new(),
                locked: false,
            })
        }
    }

    pub unsafe fn lock_unsafe(&self) -> Result<()> {
        let mut state = lock!(self.state);
        if state.locked {
            thread::block(move |thread| {
                state.waiters.push_back(thread);
            });
        } else {
            state.locked = true;
        }

        Ok(())
    }

    pub unsafe fn unlock_unsafe(&self)  -> Result<()> {
        let mut state = lock!(self.state);
        if !state.locked {
            return Err(ErrNum::NotSupported);
        }

        if let Some(thread) = state.waiters.pop_front() {
            thread.resume();
        } else {
            state.locked = false;
        }

        Ok(())
    }

    pub fn lock(&self) -> Result<UntypedMutexGuard> {
        unsafe { self.lock_unsafe()?; }
        Ok(UntypedMutexGuard::new(self))
    }
}

impl KObj for UntypedMutex {
    fn mutex(&self) -> Option<&UntypedMutex> {
        Some(self)
    }
}

#[must_use]
pub struct UntypedMutexGuard<'a> {
    lock: &'a UntypedMutex,
}

impl<'a> UntypedMutexGuard<'a> {
    pub fn new(lock: &'a UntypedMutex) -> Self {
        UntypedMutexGuard { lock }
    }
}

impl<'a> Drop for UntypedMutexGuard<'a> {
    fn drop(&mut self) {
        unsafe { self.lock.unlock_unsafe().unwrap() }
    }
}

#[cfg(feature = "test")]
pub mod test {
    use alloc::arc::Arc;
    use collections::vec::Vec;
    use core::fmt::Write;
    use logging;
    use super::*;

    test! {
        fn can_lock_single_thread() {
            thread::with_scheduler(|| {
                let m = UntypedMutex::new();
                let _g: UntypedMutexGuard = m.lock().expect("lock returned Err");
                thread::schedule();
            });
        }

        fn can_lock_lots_of_threads() {
            thread::with_scheduler(|| {
                let m = Arc::new(UntypedMutex::new());
                let mut deferreds = Vec::new();

                for i in 0..5 {
                    let m = m.clone();

                    let d = thread::spawn(move || {
                        let _ = write!(logging::Writer, ">");
                        let _g: UntypedMutexGuard = m.lock().expect("lock returned Err");
                        let _ = write!(logging::Writer, ".");
                        thread::schedule();

                        let _ = write!(logging::Writer, "<");
                        i
                    });

                    deferreds.push((i, d));
                }

                while let Some((i, d)) = deferreds.pop() {
                    assert_eq!(i, d.get());
                }

                let _ = write!(logging::Writer, "\n");
            });
        }
    }
}
