use crate::prelude::*;
use core::mem;
use core::sync::atomic::{AtomicPtr, Ordering};

pub struct Singleton<T> {
    cell: AtomicPtr<T>,
}

unsafe impl<T> Sync for Singleton<T> {}

impl<T> Singleton<T> {
    pub const fn new() -> Singleton<T> {
        Singleton {
            cell: AtomicPtr::new(0 as *mut T),
        }
    }

    pub fn get(&self) -> Option<&T> {
        let p = self.cell.load(Ordering::Relaxed);
        if p == (0 as *mut _) {
            None
        } else {
            Some(unsafe { &*p })
        }
    }

    pub fn register(&'static self, value: T) -> DropSingleton<T> {
        let b: Box<T> = Box::new(value);
        let p: *mut T = Box::into_raw(b);
        DropSingleton {
            cell: &self.cell,
            old: self.cell.swap(p, Ordering::Relaxed),
        }
    }
}

pub struct DropSingleton<T: 'static> {
    cell: &'static AtomicPtr<T>,
    old: *mut T,
}

impl<T> Drop for DropSingleton<T> {
    fn drop(&mut self) {
        let b: Box<T> = unsafe {
            let p: *mut T = self.cell.swap(self.old, Ordering::Relaxed);
            Box::from_raw(p)
        };

        mem::drop(b);
    }
}
