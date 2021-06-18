use crate::kobj::KObj;
use crate::spin::Mutex;
use crate::thread::{self, BlockedThread};
use alloc::collections::vec_deque::VecDeque;
use syscall::{ErrNum, Result};

struct SemaphoreState {
    waiters: VecDeque<BlockedThread>,
    value: usize,
}

pub struct Semaphore {
    state: Mutex<SemaphoreState>,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}

impl Semaphore {
    pub fn new(value: usize) -> Self {
        Semaphore {
            state: Mutex::new(SemaphoreState {
                waiters: VecDeque::new(),
                value,
            }),
        }
    }

    pub fn wait(&self) {
        let mut state = lock!(self.state);
        if let Some(value) = state.value.checked_sub(1) {
            state.value = value;
        } else {
            thread::block(move |thread| {
                state.waiters.push_back(thread);
            });
        }
    }

    pub fn post(&self) -> Result<()> {
        let mut state = lock!(self.state);
        if let Some(thread) = state.waiters.pop_front() {
            thread.resume();
        } else {
            state.value = state.value.checked_add(1).ok_or(ErrNum::InvalidArgument)?;
        }

        Ok(())
    }
}

impl KObj for Semaphore {
    fn semaphore(&self) -> Option<&Semaphore> {
        Some(self)
    }
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;
    use crate::logging;
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::fmt::Write;

    test! {
        fn single_thread_does_not_block() {
            thread::with_scheduler(|| {
                let s = Semaphore::new(1);
                s.wait();
                thread::schedule();
            });
        }

        fn lots_of_threads_can_block() {
            thread::with_scheduler(|| {
                let s = Arc::new(Semaphore::new(1));
                let mut deferreds = Vec::new();

                for i in 0..5 {
                    let s = s.clone();

                    let d = thread::spawn(move || {
                        let _ = write!(logging::Writer, ">");
                        s.wait();
                        let _ = write!(logging::Writer, ".");
                        thread::schedule();

                        let _ = write!(logging::Writer, "<");
                        s.post().expect("post returned Err");
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
