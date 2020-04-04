use super::{OSHandle, Result};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use syscall;

pub struct Mutex<T: ?Sized> {
    handle: OSHandle,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Result<Self> {
        Ok(Mutex {
            handle: OSHandle::from_raw(syscall::create_mutex()?),
            data: UnsafeCell::new(data),
        })
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn handle(&self) -> &OSHandle {
        &self.handle
    }

    pub fn lock(&self) -> Result<MutexGuard<T>> {
        syscall::lock_mutex(self.handle().get())?;
        Ok(MutexGuard::new(self))
    }
}

#[must_use]
pub struct MutexGuard<'mutex, T: ?Sized + 'mutex> {
    lock: &'mutex Mutex<T>,
    _pd: PhantomData<*mut T>,
}

unsafe impl<'mutex, T: ?Sized + Sync> Sync for MutexGuard<'mutex, T> {}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    fn new(lock: &'mutex Mutex<T>) -> Self {
        MutexGuard { lock, _pd: PhantomData }
    }
}

impl<'mutex, T: ?Sized> Drop for MutexGuard<'mutex, T> {
    fn drop(&mut self) {
        let _ = syscall::unlock_mutex(self.lock.handle().get());
    }
}

impl<'mutex, T: ?Sized> Deref for MutexGuard<'mutex, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'mutex, T: ?Sized> DerefMut for MutexGuard<'mutex, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
