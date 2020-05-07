use alloc::rc::Rc;
use alloc::string::String;
use cairo::cairo::Cairo;
use graphics_base::types::{Color, EventInput, Rect};
use graphics_base::Result;
use hecs::{Entity, World};

#[derive(Clone)]
pub struct BackColor(pub Color);

impl BackColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Color { r, g, b })
    }
}

#[derive(Clone)]
pub struct CapturesMouseInput;

pub struct Focus(pub Option<Entity>);

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

impl Position {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self(Rect { x, y, width, height })
    }
}

#[derive(Clone)]
pub struct Text(pub String);

impl Text {
    pub fn new<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        Self(s.into())
    }
}

#[derive(Clone)]
pub struct TextColor(pub Color);

impl TextColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self(Color { r, g, b })
    }
}
