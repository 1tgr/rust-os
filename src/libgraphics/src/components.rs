use crate::compat::Cairo;
use crate::types::{EventInput, Rect};
use crate::Result;
use alloc::rc::Rc;
use hecs::{Entity, World};

#[derive(Clone, PartialEq)]
pub struct Position(pub Rect);

pub struct OnPaint(pub Rc<dyn Fn(&World, Entity, &Cairo)>);

impl OnPaint {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&World, Entity, &Cairo) + 'static,
    {
        Self(Rc::new(f))
    }
}

pub struct OnInput(pub Rc<dyn Fn(&mut World, Entity, &EventInput) -> Result<()>>);

impl OnInput {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut World, Entity, &EventInput) -> Result<()> + 'static,
    {
        Self(Rc::new(f))
    }
}

pub struct NeedsPaint;
