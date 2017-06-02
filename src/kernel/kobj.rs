use alloc::arc::Arc;
use core::mem;
use core::nonzero::NonZero;
use core::ops::Deref;
use deferred::Deferred;
use io::{AsyncRead,Read,Write};
use mutex::UntypedMutex;
use process::{Process,SharedMemBlock};
use syscall::{ErrNum,Result};

pub trait KObj {
    fn async_read(&self) -> Option<&AsyncRead> { None }
    fn read(&self) -> Option<&Read> { None }
    fn write(&self) -> Option<&Write> { None }
    fn deferred_i32(&self) -> Option<Deferred<i32>> { None }
    fn shared_mem_block(&self) -> Option<&SharedMemBlock> { None }
    fn process(&self) -> Option<&Process> { None }
    fn mutex(&self) -> Option<&UntypedMutex> { None }
}

pub struct KObjRef<T: ?Sized> {
    kobj: Arc<KObj>,
    ptr: NonZero<*const T>
}

impl<'a, T: ?Sized+'a> KObjRef<T> {
    pub fn new<F: FnOnce(&'a KObj) -> Option<&'a T>>(kobj: Arc<KObj>, f: F) -> Result<Self> {
        let ptr = {
            let kobj: &KObj = &*kobj;
            let kobj: &'a KObj = unsafe { mem::transmute(kobj) };
            match f(kobj) {
                Some(r) => r as *const T,
                None => { return Err(ErrNum::NotSupported) }
            }
        };

        Ok(KObjRef { kobj: kobj, ptr: unsafe { NonZero::new(ptr) } })
    }

    pub fn get(&self) -> &Arc<KObj> {
        &self.kobj
    }
}

impl<T: ?Sized> Clone for KObjRef<T> {
    fn clone(&self) -> Self {
        KObjRef { kobj: self.kobj.clone(), ptr: self.ptr }
    }
}

impl<T: ?Sized> Deref for KObjRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = self.ptr.get();
        unsafe { &*ptr }
    }
}
