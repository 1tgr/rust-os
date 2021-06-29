use alloc::sync::Arc;
use euclid::Size2D;
use os::OSHandle;
use ui::property_map::Property;
use ui_types::types::ScreenSpace;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SharedMemHandle;

impl Property for SharedMemHandle {
    type Value = Arc<OSHandle>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct PixelSize;

impl Property for PixelSize {
    type Value = Size2D<i32, ScreenSpace>;
}
