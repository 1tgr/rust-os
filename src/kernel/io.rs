use prelude::*;
use thread::Deferred;

trait PromiseNode<A> {
    fn get(self: Box<Self>) -> A;
    fn try_get(self: Box<Self>) -> Result<A, Box<PromiseNode<A>>>;
}

struct DeferredNode<A> {
    d: Deferred<A>
}

impl<A> PromiseNode<A> for DeferredNode<A> where A: 'static {
    fn get(self: Box<Self>) -> A {
        self.d.get()
    }

    fn try_get(self: Box<Self>) -> Result<A, Box<PromiseNode<A>>> {
        self.d.try_get().map_err(|d| {
            let b: Box<PromiseNode<A>> = Box::new(DeferredNode { d: d });
            b
        })
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

    fn try_get(self: Box<Self>) -> Result<B, Box<PromiseNode<B>>> {
        let p = *self;
        match p.node.try_get() {
            Ok(result) => Ok((p.f)(result)),
            Err(node) => Err(Box::new(MapNode { node: node, f: p.f }))
        }
    }
}

pub struct Promise<A> {
    node: Box<PromiseNode<A>>
}

impl<A: 'static> Promise<A> {
    pub fn new(d: Deferred<A>) -> Self {
        Promise { node: Box::new(DeferredNode { d: d }) }
    }

    pub fn get(self) -> A {
        self.node.get()
    }

    pub fn try_get(self) -> Result<A, Self> {
        self.node.try_get().map_err(|node| Promise { node: node })
    }

    pub fn then<B, F>(self, f: F) -> Promise<B> where F: FnOnce(A) -> B {
        unimplemented!()
    }
}

pub trait AsyncRead {
    fn read_async(&self, buf: Vec<u8>) -> Promise<Result<(Vec<u8>, usize), &'static str>>;
}
