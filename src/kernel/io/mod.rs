mod nodes;

use core::result;
use core::slice::bytes;
use io::nodes::PromiseNode;
use prelude::*;
use syscall::Result;
use thread::Deferred;

pub trait Read {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
}

pub struct Promise<A>(Box<PromiseNode<A>>);

impl<A: 'static> Promise<A> {
    pub fn new(d: Deferred<A>) -> Self {
        Promise(Box::new(nodes::deferred(d)))
    }

    pub fn resolved(value: A) -> Self {
        Promise(Box::new(nodes::resolved(value)))
    }

    pub fn get(self) -> A {
        self.0.get()
    }

    pub fn try_get(self) -> result::Result<A, Self> {
        self.0.try_get().map_err(|node| Promise(node))
    }

    pub fn then<B, F: FnOnce(A) -> B + 'static>(self, f: F) -> Promise<B> {
        Promise(Box::new(self.0.map(f)))
    }
}

impl<A: 'static> Promise<Promise<A>> {
    pub fn unwrap(self) -> Promise<A> {
        Promise(Box::new(self.0.unwrap()))
    }
}

pub trait AsyncRead {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<(Vec<u8>, usize)>>;
}

impl<T: AsyncRead> Read for T {
    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        let p = self.read_async(vec![0; buf.len()]);
        let (v, len) = try!(p.get());
        bytes::copy_memory(&v[..], buf);
        Ok(len)
    }
}
