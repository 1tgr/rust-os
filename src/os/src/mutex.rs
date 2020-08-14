use crate::detail::UntypedMutex;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub struct Mutex<T: ?Sized> {
    mutex: UntypedMutex,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            mutex: UntypedMutex::new(),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<T> {
        self.mutex.lock();
        MutexGuard {
            mutex: self,
            _pd: PhantomData,
        }
    }
}

#[must_use]
pub struct MutexGuard<'mutex, T: ?Sized + 'mutex> {
    mutex: &'mutex Mutex<T>,
    _pd: PhantomData<*mut T>,
}

unsafe impl<'mutex, T: ?Sized + Sync> Sync for MutexGuard<'mutex, T> {}

impl<'mutex, T: ?Sized> Drop for MutexGuard<'mutex, T> {
    fn drop(&mut self) {
        self.mutex.mutex.unlock();
    }
}

impl<'mutex, T: ?Sized> Deref for MutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}
