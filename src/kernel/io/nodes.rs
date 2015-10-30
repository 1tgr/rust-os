use core::result;
use deferred::Deferred;
use io::Promise;
use prelude::*;

pub trait PromiseNode<A> {
    fn get(self: Box<Self>) -> A;
    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>>;
    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>>;
}

pub struct DeferredNode<A>(Deferred<A>);
pub struct ResolvedNode<A>(A);
pub struct MapNode<A, F>(Box<PromiseNode<A>>, F);
pub struct UnwrapNode<A>(Box<PromiseNode<Promise<A>>>);

pub fn deferred<A>(d: Deferred<A>) -> DeferredNode<A> {
    DeferredNode(d)
}

pub fn resolved<A>(value: A) -> ResolvedNode<A> {
    ResolvedNode(value)
}

impl<A: 'static> PromiseNode<A> {
    pub fn map<B, F: FnOnce(A) -> B + 'static>(self: Box<Self>, f: F) -> MapNode<A, F> {
        MapNode(self, f)
    }
}

impl<A: 'static> PromiseNode<Promise<A>> {
    pub fn unwrap(self: Box<Self>) -> UnwrapNode<A> {
        UnwrapNode(self)
    }
}

impl<A: 'static> PromiseNode<A> for DeferredNode<A> {
    fn get(self: Box<Self>) -> A {
        self.0.get()
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        self.0.try_get().map_err(|d| {
            let b: Box<PromiseNode<A>> = Box::new(DeferredNode(d));
            b
        })
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Err(self)
    }
}

impl<A: 'static, B, F: FnOnce(A) -> B+'static> PromiseNode<B> for MapNode<A, F> {
    fn get(self: Box<Self>) -> B {
        let p = *self;
        p.1(p.0.get())
    }

    fn try_get(self: Box<Self>) -> result::Result<B, Box<PromiseNode<B>>> {
        let p = *self;
        match p.0.try_get() {
            Ok(result) => Ok(p.1(result)),
            Err(node) => Err(Box::new(MapNode(node, p.1)))
        }
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<B>, Box<PromiseNode<B>>> {
        Err(self)
    }
}

impl<A: 'static> PromiseNode<A> for UnwrapNode<A> {
    fn get(self: Box<Self>) -> A {
        let mut p: Promise<A> = self.0.get();
        loop {
            match p.0.try_unwrap() {
                Ok(inner) => p = inner,
                Err(node) => { return node.get() }
            }
        }
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        match self.0.try_get() {
            Ok(node) => node.try_get().map_err(|p| p.0),
            Err(node) => Err(Box::new(UnwrapNode(node)))
        }
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Ok(self.0.get())
    }
}

impl<A: 'static> PromiseNode<A> for ResolvedNode<A> {
    fn get(self: Box<Self>) -> A {
        self.0
    }

    fn try_get(self: Box<Self>) -> result::Result<A, Box<PromiseNode<A>>> {
        Ok(self.0)
    }

    fn try_unwrap(self: Box<Self>) -> result::Result<Promise<A>, Box<PromiseNode<A>>> {
        Err(self)
    }
}
