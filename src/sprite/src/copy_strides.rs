use core::convert::TryInto;
use euclid::{Box2D, Point2D, Size2D};

#[derive(Debug, Clone, PartialEq)]
pub struct CopyStrides {
    dst_start: usize,
    dst_stride: usize,
    src_start: usize,
    src_stride: usize,
    prefix_per_stride: usize,
    stride_count: usize,
}

impl CopyStrides {
    pub fn init<SrcSpace, DstSpace>(
        dst_size: Size2D<i32, DstSpace>,
        src_size: Size2D<i32, SrcSpace>,
        mut src_rect: Box2D<i32, SrcSpace>,
        mut dst_pos: Point2D<i32, DstSpace>,
    ) -> Option<Self> {
        if dst_pos.x < 0 {
            src_rect.min.x -= dst_pos.x;
            dst_pos.x = 0;
        }

        if dst_pos.y < 0 {
            src_rect.min.y -= dst_pos.y;
            dst_pos.y = 0;
        }

        let dst_bounds = Box2D::from_size(dst_size);
        let src_bounds = Box2D::from_size(src_size);
        let src_rect = src_rect.intersection_unchecked(&src_bounds);
        let v = dst_pos - src_rect.min.cast_unit::<DstSpace>();

        let src_rect = src_rect
            .cast_unit::<DstSpace>()
            .translate(v)
            .intersection_unchecked(&dst_bounds)
            .translate(-v)
            .cast_unit::<SrcSpace>();

        if src_rect.is_empty() {
            return None;
        }

        let dst_pos = Point2D::<i32, DstSpace>::new(dst_pos.x.min(dst_bounds.max.x), dst_pos.y.min(dst_bounds.max.y));

        let src_rect = src_rect.try_cast::<usize>().unwrap();
        let dst_pos = dst_pos.try_cast::<usize>().unwrap();
        let dst_stride = dst_size.width.try_into().unwrap();
        let src_stride = src_size.width.try_into().unwrap();
        let dst_start = dst_pos.y * dst_stride + dst_pos.x;
        let src_start = src_rect.min.y * src_stride + src_rect.min.x;
        let prefix_per_stride = src_rect.width();
        let stride_count = src_rect.height();
        Some(Self {
            dst_start,
            dst_stride,
            src_start,
            src_stride,
            prefix_per_stride,
            stride_count,
        })
    }

    pub fn blend<Blend>(&self, dst: &mut [u32], src: &[u32])
    where
        Blend: sw_composite::blend::Blend,
    {
        let dst = &mut dst[self.dst_start..];
        let src = &src[self.src_start..];
        for (dst_chunk, src_chunk) in dst
            .chunks_exact_mut(self.dst_stride)
            .zip(src.chunks_exact(self.src_stride))
            .take(self.stride_count)
        {
            for (dst, &src) in dst_chunk.iter_mut().zip(src_chunk.iter()).take(self.prefix_per_stride) {
                *dst = Blend::blend(src, *dst);
            }
        }
    }

    pub fn blend_a8(&self, dst: &mut [u32], src: u32, alpha: &[u8]) {
        let dst = &mut dst[self.dst_start..];
        let alpha = &alpha[self.src_start..];
        for (dst_chunk, alpha_chunk) in dst
            .chunks_exact_mut(self.dst_stride)
            .zip(alpha.chunks_exact(self.src_stride))
            .take(self.stride_count)
        {
            for (dst, &alpha) in dst_chunk
                .iter_mut()
                .zip(alpha_chunk.iter())
                .take(self.prefix_per_stride)
            {
                *dst = sw_composite::over_in(src, *dst, alpha as u32);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::copy_strides::CopyStrides;
    use euclid::default::{Box2D, Point2D, Size2D};

    #[test]
    fn small_over_big() {
        let dst_size = Size2D::new(300, 300);
        let src_size = Size2D::new(100, 100);
        let dst_pos = Point2D::new(100, 100);
        let dst_stride = dst_size.width as usize;
        let src_stride = src_size.width as usize;
        let c = CopyStrides::init(dst_size, src_size, Box2D::from_size(src_size), dst_pos).unwrap();
        assert_eq!(
            c,
            CopyStrides {
                dst_start: dst_pos.x as usize + dst_stride * (dst_pos.y as usize),
                dst_stride,
                src_start: 0,
                src_stride,
                prefix_per_stride: src_stride,
                stride_count: src_size.height as usize,
            }
        );
    }

    #[test]
    fn big_over_small() {
        let dst_size = Size2D::new(100, 100);
        let src_size = Size2D::new(300, 300);
        let dst_pos = Point2D::new(-100, -100);
        let dst_stride = dst_size.width as usize;
        let src_stride = src_size.width as usize;
        let c = CopyStrides::init(dst_size, src_size, Box2D::from_size(src_size), dst_pos).unwrap();
        assert_eq!(
            c,
            CopyStrides {
                dst_start: 0,
                dst_stride,
                src_start: (-dst_pos.x as usize) + src_stride * (-dst_pos.y as usize),
                src_stride,
                prefix_per_stride: dst_stride,
                stride_count: dst_size.height as usize,
            }
        );
    }

    #[test]
    fn pointer_bottom_right() {
        let dst_size = Size2D::new(800, 600);
        let src_size = Size2D::new(800, 600);
        let dst_pos = Point2D::new(500, 400);
        let dst_stride = dst_size.width as usize;
        let src_stride = src_size.width as usize;
        let src_rect = Box2D::from_origin_and_size(dst_pos, Size2D::new(32, 32));
        let c = CopyStrides::init(dst_size, src_size, src_rect, dst_pos).unwrap();
        assert_eq!(
            c,
            CopyStrides {
                dst_start: dst_pos.x as usize + dst_stride * (dst_pos.y as usize),
                dst_stride,
                src_start: dst_pos.x as usize + dst_stride * (dst_pos.y as usize),
                src_stride,
                prefix_per_stride: src_rect.width() as usize,
                stride_count: src_rect.height() as usize,
            }
        );
    }

    #[test]
    fn pointer_top_left() {
        let dst_size = Size2D::new(800, 600);
        let src_size = Size2D::new(800, 600);
        let dst_pos = Point2D::new(-16, -16);
        let dst_stride = dst_size.width as usize;
        let src_stride = src_size.width as usize;
        let src_rect = Box2D::from_origin_and_size(dst_pos, Size2D::new(32, 32));
        let c = CopyStrides::init(dst_size, src_size, src_rect, dst_pos).unwrap();
        assert_eq!(
            c,
            CopyStrides {
                dst_start: 0,
                dst_stride,
                src_start: 0,
                src_stride,
                prefix_per_stride: (src_rect.width() + dst_pos.x) as usize,
                stride_count: (src_rect.height() + dst_pos.y) as usize,
            }
        );
    }
}
