use core::result;
use core::slice::bytes;
use prelude::*;
use syscall::Result;
use thread::Deferred;

pub trait Read {
    fn read(&self, buf: &mut [u8]) -> Result<usize>;
}

trait PromiseNode<A> {
    fn get(self: Box<Self>) -> A;
    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>>;
    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>>;
}

struct DeferredNode<A> {
    d: Deferred<A>
}

impl<A> PromiseNode<A> for DeferredNode<A> where A: 'static {
    fn get(self: Box<Self>) -> A {
        self.d.get()
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        self.d.try_get().map_err(|d| {
            let b: Box<PromiseNode<A>> = Box::new(DeferredNode { d: d });
            b
        })
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Err(self)
    }
}

struct MapNode<A, F> {
    node: Box<PromiseNode<A>>,
    f: F
}

impl<A: 'static, B, F: FnOnce(A) -> B+'static> PromiseNode<B> for MapNode<A, F> {
    fn get(self: Box<Self>) -> B {
        let p = *self;
        (p.f)(p.node.get())
    }

    fn try_get(self: Box<Self>) -> result::Result<B, Box<PromiseNode<B>>> {
        let p = *self;
        match p.node.try_get() {
            Ok(result) => Ok((p.f)(result)),
            Err(node) => Err(Box::new(MapNode { node: node, f: p.f }))
        }
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<B>, Box<PromiseNode<B>>> {
        Err(self)
    }
}

struct UnwrapNode<A> {
    node: Box<PromiseNode<Promise<A>>>
}

impl<A: 'static> PromiseNode<A> for UnwrapNode<A> {
    fn get(self: Box<Self>) -> A {
        let mut p: Promise<A> = self.node.get();
        loop {
            match p.node.try_unwrap() {
                Ok(inner) => p = inner,
                Err(node) => { return node.get() }
            }
        }
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        match (*self).node.try_get() {
            Ok(node) => node.try_get().map_err(|p| p.node),
            Err(node) => Err(Box::new(UnwrapNode { node: node }))
        }
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Ok(self.node.get())
    }
}

struct ResolvedNode<A> {
    value: A
}

impl<A: 'static> PromiseNode<A> for ResolvedNode<A> {
    fn get(self: Box<Self>) -> A {
        let p = *self;
        p.value
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        let p = *self;
        Ok(p.value)
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Err(self)
    }
}

pub struct Promise<A> {
    node: Box<PromiseNode<A>>
}

impl<A: 'static> Promise<A> {
    pub fn new(d: Deferred<A>) -> Self {
        Promise { node: Box::new(DeferredNode { d: d }) }
    }

    pub fn resolved(value: A) -> Self {
        Promise { node: Box::new(ResolvedNode { value: value }) }
    }

    pub fn get(self) -> A {
        self.node.get()
    }

    pub fn try_get(self) -> result::Result<A, Self> {
        self.node.try_get().map_err(|node| Promise { node: node })
    }

    pub fn then<B, F: FnOnce(A) -> B + 'static>(self, f: F) -> Promise<B> {
        Promise { node: Box::new(MapNode { node: self.node, f: f }) }
    }
}

impl<A: 'static> Promise<Promise<A>> {
    pub fn unwrap(self) -> Promise<A> {
        Promise { node: Box::new(UnwrapNode { node: self.node }) }
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
