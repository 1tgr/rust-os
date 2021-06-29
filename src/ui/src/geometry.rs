use crate::path::Path;
use euclid::{Point2D, Size2D, Transform2D};
use ui_types::types::ScreenSpace;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ObjectSpace {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParentSpace {}

pub type ObjectSize = Size2D<f32, ObjectSpace>;

pub type ScreenPath = Path<f32, ScreenSpace>;
pub type ScreenPoint = Point2D<f32, ScreenSpace>;
pub type ScreenTransform = Transform2D<f32, ObjectSpace, ScreenSpace>;

pub type ParentPoint = Point2D<f32, ParentSpace>;
pub type ParentTransform = Transform2D<f32, ObjectSpace, ParentSpace>;
