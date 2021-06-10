use euclid::{Box2D, Point2D, Size2D, Vector2D};
use sprite::Sprite;
use sw_composite::blend::{Src, SrcOver};
use syscall::ErrNum;
use ui_types::types::ScreenSpace;
use ui_types::Result;

static CURSOR_PNG: &[u8] = include_bytes!("icons8-cursor-32.png");
const CURSOR_HOTSPOT: Vector2D<i32, ScreenSpace> = Vector2D::new(12, 8);

#[derive(Clone)]
pub struct MousePointer {
    pub pos: Point2D<i32, ScreenSpace>,
    bounds: Box2D<i32, ScreenSpace>,
    over: Sprite<Vec<u32>, ScreenSpace>,
    under: Sprite<Vec<u32>, ScreenSpace>,
}

impl MousePointer {
    pub fn init(screen_size: Size2D<i32, ScreenSpace>) -> Result<Self> {
        let bounds = Box2D::from_size(screen_size);

        let over = {
            let (header, data) = png_decoder::decode(CURSOR_PNG).map_err(|_| ErrNum::NotSupported)?;
            let size = Size2D::new(header.width as i32, header.height as i32);
            let (ptr, len, capacity): (*mut u8, usize, usize) = data.into_raw_parts();
            let mut data = unsafe { Vec::from_raw_parts(ptr as *mut u32, len / 4, capacity / 4) };
            for x in data.iter_mut() {
                *x = sw_composite::alpha_mul(*x, *x >> 24);
            }

            Sprite::from_backing(size, data)
        };

        let under = Sprite::new(over.size);

        Ok(Self {
            pos: bounds.center(),
            bounds,
            over,
            under,
        })
    }

    pub fn update_delta<Backing1, Backing2>(
        &mut self,
        lfb_back: &mut Sprite<Backing1, ScreenSpace>,
        lfb: &mut Sprite<Backing2, ScreenSpace>,
        delta: Vector2D<i32, ScreenSpace>,
    ) where
        Backing1: AsRef<[u32]> + AsMut<[u32]>,
        Backing2: AsMut<[u32]>,
    {
        // 1. Restore the area under the old mouse pointer
        let prev_rect = self.pointer_rect();
        lfb_back.draw_sprite_at(&self.under, prev_rect.min, Src);

        // 2. Update the mouse pointer position
        self.pos = (self.pos + delta).clamp(self.bounds.min, self.bounds.max);

        // 3. Save the image under the new mouse pointer and draw the new mouse pointer
        let rect = self.pointer_rect();
        self.draw(lfb_back);

        // 4. Copy the restored and new areas to the front buffer
        if prev_rect.intersects(&rect) {
            let rect = prev_rect.union(&rect);
            lfb.draw_sprite_region_at(lfb_back, rect, rect.min, Src);
        } else {
            lfb.draw_sprite_region_at(lfb_back, prev_rect, prev_rect.min, Src);
            lfb.draw_sprite_region_at(lfb_back, rect, rect.min, Src);
        }
    }

    pub fn pointer_rect(&self) -> Box2D<i32, ScreenSpace> {
        Box2D::from_origin_and_size(self.pos - CURSOR_HOTSPOT, self.over.size)
    }

    pub fn draw<Backing>(&mut self, lfb_back: &mut Sprite<Backing, ScreenSpace>)
    where
        Backing: AsRef<[u32]> + AsMut<[u32]>,
    {
        let dst_pos = self.pos - CURSOR_HOTSPOT;
        self.under.draw_sprite_at(lfb_back, -dst_pos, Src);
        lfb_back.draw_sprite_at(&self.over, dst_pos, SrcOver);
    }
}
