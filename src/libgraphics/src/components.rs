use crate::types::{EventInput, Rect};
use crate::Result;
use alloc::rc::Rc;
use cairo::cairo::Cairo;
use hecs::{Entity, World};

#[derive(Clone)]
pub struct CapturesMouseInput;

#[derive(Clone)]
pub struct OnClick(pub Rc<dyn Fn(&mut World, Entity) -> Result<()>>);

impl OnClick {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut World, Entity) -> Result<()> + 'static,
    {
        Self(Rc::new(f))
    }
}

#[derive(Clone)]
pub struct OnInput(pub Rc<dyn Fn(&mut World, Entity, EventInput) -> Result<()>>);

impl OnInput {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut World, Entity, EventInput) -> Result<()> + 'static,
    {
        Self(Rc::new(f))
    }
}

#[derive(Clone)]
pub struct OnPaint(pub Rc<dyn Fn(&World, Entity, &Cairo)>);

impl OnPaint {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&World, Entity, &Cairo) + 'static,
    {
        Self(Rc::new(f))
    }
}

#[derive(Clone)]
pub struct NeedsPaint;

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Parent(pub Entity);

#[derive(Clone, PartialEq)]
pub struct Position(pub Rect);

#[derive(Clone)]
pub struct Text(pub String);

impl Text {
    pub fn new<S>(s: S) -> Self where S: Into<String> {
        Self(s.into())
    }
}
