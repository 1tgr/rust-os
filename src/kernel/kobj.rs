use crate::deferred::Deferred;
use crate::io::{AsyncRead, Read, Write};
use crate::mutex::UntypedMutex;
use crate::process::{Process, SharedMemBlock};
use alloc::sync::Arc;
use core::mem;
use core::ops::Deref;
use syscall::{ErrNum, Result};

pub trait KObj {
    fn async_read(&self) -> Option<&dyn AsyncRead> {
        None
    }
    fn read(&self) -> Option<&dyn Read> {
        None
    }
    fn write(&self) -> Option<&dyn Write> {
        None
    }
    fn deferred_i32(&self) -> Option<Deferred<i32>> {
        None
    }
    fn shared_mem_block(&self) -> Option<&SharedMemBlock> {
        None
    }
    fn process(&self) -> Option<&Process> {
        None
    }
    fn mutex(&self) -> Option<&UntypedMutex> {
        None
    }
}

pub struct KObjRef<T: ?Sized> {
    kobj: Arc<dyn KObj>,
    ptr: *const T,
}

impl<'a, T: ?Sized + 'a> KObjRef<T> {
    pub fn new<F: FnOnce(&'a dyn KObj) -> Option<&'a T>>(kobj: Arc<dyn KObj>, f: F) -> Result<Self> {
        let ptr = {
            let kobj: &dyn KObj = &*kobj;
            let kobj: &'a dyn KObj = unsafe { mem::transmute(kobj) };
            match f(kobj) {
                Some(r) => r as *const T,
                None => return Err(ErrNum::NotSupported),
            }
        };

        Ok(KObjRef { kobj, ptr })
    }

    pub fn get(&self) -> &Arc<dyn KObj> {
        &self.kobj
    }
}

impl<T: ?Sized> Clone for KObjRef<T> {
    fn clone(&self) -> Self {
        KObjRef {
            kobj: self.kobj.clone(),
            ptr: self.ptr,
        }
    }
}

impl<T: ?Sized> Deref for KObjRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}
