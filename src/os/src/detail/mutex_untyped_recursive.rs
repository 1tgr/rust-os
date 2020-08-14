use crate::{OSHandle, Thread};
use core::cell::{Cell, UnsafeCell};
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct UntypedRecursiveMutex {
    counter: AtomicUsize,
    owner: UnsafeCell<usize>,
    recursion: Cell<usize>,
    handle: OSHandle,
}

unsafe impl Send for UntypedRecursiveMutex {}
unsafe impl Sync for UntypedRecursiveMutex {}

impl UntypedRecursiveMutex {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            owner: UnsafeCell::new(0),
            recursion: Cell::new(0),
            handle: OSHandle::from_raw(syscall::create_mutex()),
        }
    }
}

impl UntypedRecursiveMutex {
    pub fn lock(&self) {
        let current_thread_id = Thread::current_thread_id();
        let owner = self.owner.get();

        if self.counter.fetch_add(1, Ordering::SeqCst) > 0 {
            if unsafe { owner.read_volatile() } != current_thread_id {
                syscall::lock_mutex(self.handle.get()).unwrap();
            }
        }

        unsafe {
            owner.write_volatile(current_thread_id);
        }

        self.recursion.set(self.recursion.get() + 1);
    }

    pub fn unlock(&self) {
        let current_thread_id = Thread::current_thread_id();
        let owner = self.owner.get();

        let release_ownership = {
            unsafe {
                assert_eq!(owner.read_volatile(), current_thread_id);
            }

            let recursion = self.recursion.get();
            if recursion == 1 {
                unsafe {
                    owner.write_volatile(0);
                }
                self.recursion.set(0);
                true
            } else {
                self.recursion.set(recursion - 1);
                false
            }
        };

        if self.counter.fetch_sub(1, Ordering::SeqCst) > 1 {
            if release_ownership {
                syscall::unlock_mutex(self.handle.get()).unwrap();
            }
        }
    }
}
