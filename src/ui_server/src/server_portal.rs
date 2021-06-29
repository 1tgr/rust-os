use crate::property::{PixelSize, SharedMemHandle};
use alloc::sync::Arc;
use core::mem;
use euclid::{Box2D, Point2D, Size2D};
use os::{OSHandle, SharedMem};
use raqote::{BlendMode, DrawOptions};
use sprite::Sprite;
use ui::prelude::*;
use ui::render::{DrawTarget, RenderDb, RenderState};
use ui_types::types::ScreenSpace;

#[derive(Clone, PartialEq)]
pub struct ServerPortalRenderState {
    origin: Point2D<f32, ScreenSpace>,
    pixel_size: Size2D<i32, ScreenSpace>,
    shared_mem_handle: Arc<OSHandle>,
}

impl HasProperty<Origin> for ServerPortal {}
impl HasProperty<PixelSize> for ServerPortal {}
impl HasProperty<SharedMemHandle> for ServerPortal {}

impl RenderState for ServerPortalRenderState {
    fn build(db: &dyn RenderDb, widget_id: WidgetId) -> Self {
        let properties = db.properties();
        let origin = db.origin(widget_id).unwrap_or_default().cast_unit();
        let shared_mem_handle = properties.get(widget_id, &SharedMemHandle).cloned().unwrap();

        let pixel_size = properties
            .get(widget_id, &PixelSize)
            .copied()
            .unwrap_or_else(Size2D::zero);

        Self {
            origin,
            shared_mem_handle,
            pixel_size,
        }
    }

    fn bounds(&self) -> Box2D<f32, ScreenSpace> {
        Box2D::from_origin_and_size(self.origin, self.pixel_size.cast())
    }

    fn render_to(&self, target: &mut DrawTarget) {
        let Self {
            origin,
            pixel_size,
            ref shared_mem_handle,
        } = *self;

        let mut shared_mem = SharedMem::from_raw(
            OSHandle::from_raw(shared_mem_handle.get()),
            pixel_size.width as usize * pixel_size.height as usize,
            false,
        )
        .unwrap();

        let src = Sprite::from_backing(pixel_size, shared_mem.as_mut());
        target.draw_image_at(
            origin.x,
            origin.y,
            &src.as_image(),
            &DrawOptions {
                blend_mode: BlendMode::Src,
                ..DrawOptions::new()
            },
        );

        let (shared_mem_handle, _) = shared_mem.into_inner();
        mem::forget(shared_mem_handle);
    }
}

#[derive(Default, PartialEq, Hash)]
pub struct ServerPortal;

impl Widget for ServerPortal {
    type RenderState = ServerPortalRenderState;
}
