use crate::copy_strides::CopyStrides;
use crate::fill_strides::FillStrides;
use alloc::vec::Vec;
use euclid::{Box2D, Point2D, Size2D, UnknownUnit};
use fontdue::layout::{CoordinateSystem, GlyphPosition, GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use fontdue::Font;
use raqote::{DrawTarget, SolidSource};
use sw_composite::Image;

#[derive(Clone)]
pub struct Sprite<Backing, Space> {
    pub size: Size2D<i32, Space>,
    pub data: Backing,
}

impl<Backing, Space> Sprite<Backing, Space>
where
    Backing: AsRef<[u32]>,
{
    pub fn from_backing(size: Size2D<i32, Space>, data: Backing) -> Self {
        let len = size.width as usize * size.height as usize;
        assert_eq!(data.as_ref().len(), len);
        Self { size, data }
    }

    pub fn as_image(&self) -> Image {
        Image {
            width: self.size.width,
            height: self.size.height,
            data: self.data.as_ref(),
        }
    }
}

impl<'a, Space> Sprite<&'a mut [u32], Space> {
    pub fn from_draw_target<Backing>(target: &'a mut DrawTarget<Backing>) -> Self
    where
        Backing: AsMut<[u32]>,
    {
        Self::from_backing(Size2D::new(target.width(), target.height()), target.get_data_mut())
    }
}

impl<Backing, Space> Sprite<Backing, Space>
where
    Backing: AsMut<[u32]>,
{
    pub fn as_draw_target_mut(&mut self) -> DrawTarget<&mut [u32]> {
        DrawTarget::from_backing(self.size.width, self.size.height, self.data.as_mut())
    }

    pub fn draw_sprite_at<Blend, SrcBacking, SrcSpace>(
        &mut self,
        src: &Sprite<SrcBacking, SrcSpace>,
        dst_pos: Point2D<i32, Space>,
        blend: Blend,
    ) where
        SrcBacking: AsRef<[u32]>,
        Blend: sw_composite::blend::Blend,
    {
        self.draw_sprite_region_at(src, Box2D::from_size(src.size), dst_pos, blend);
    }

    pub fn draw_sprite_region_at<Blend, SrcBacking, SrcSpace>(
        &mut self,
        src: &Sprite<SrcBacking, SrcSpace>,
        src_rect: Box2D<i32, SrcSpace>,
        dst_pos: Point2D<i32, Space>,
        _blend: Blend,
    ) where
        SrcBacking: AsRef<[u32]>,
        Blend: sw_composite::blend::Blend,
    {
        if let Some(c) = CopyStrides::init(self.size, src.size, src_rect, dst_pos) {
            c.blend::<Blend>(self.data.as_mut(), src.data.as_ref());
        }
    }

    pub fn fill_rect<Blend>(&mut self, rect: Box2D<i32, Space>, color: SolidSource, _blend: Blend)
    where
        Blend: sw_composite::blend::Blend,
    {
        if let Some(f) = FillStrides::init(self.size, rect) {
            f.fill::<Blend>(self.data.as_mut(), color.to_u32());
        }
    }
}

impl<Backing> Sprite<Backing, UnknownUnit>
where
    Backing: AsMut<[u32]>,
{
    pub fn draw_text(&mut self, font: &Font, px: f32, color: SolidSource, settings: &LayoutSettings, text: &str) {
        let color = color.to_u32();
        let fonts = &[font];
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(settings);
        layout.append(fonts, &TextStyle::new(text, px, 0));
        for g in layout.glyphs() {
            let &GlyphPosition {
                key:
                    GlyphRasterConfig {
                        glyph_index,
                        px,
                        font_index,
                    },
                x,
                y,
                width,
                height,
                char_data: _,
                user_data: (),
            } = g;
            assert_eq!(font_index, 0);

            let (_, glyph) = font.rasterize_indexed(glyph_index as usize, px);
            let src_size = Size2D::<usize, UnknownUnit>::new(width, height).to_i32();
            let src_rect = Box2D::from_size(src_size);
            let dst_pos = Point2D::new(x, y).to_i32();
            if let Some(c) = CopyStrides::init(self.size, src_size, src_rect, dst_pos) {
                c.blend_a8(self.data.as_mut(), color, &glyph);
            }
        }
    }
}

impl<Space> Sprite<Vec<u32>, Space> {
    pub fn new(size: Size2D<i32, Space>) -> Self {
        let len = size.width as usize * size.height as usize;
        Self {
            size,
            data: vec![0; len],
        }
    }
}
