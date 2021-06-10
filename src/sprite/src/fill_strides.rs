use core::convert::TryInto;
use euclid::{Box2D, Size2D};

#[derive(Debug, Clone, PartialEq)]
pub struct FillStrides {
    start: usize,
    stride: usize,
    prefix_per_stride: usize,
    stride_count: usize,
}

impl FillStrides {
    pub fn init<Space>(dst_size: Size2D<i32, Space>, rect: Box2D<i32, Space>) -> Option<Self> {
        let dst_rect = Box2D::from_size(dst_size);
        let rect = dst_rect.intersection(&rect)?;
        let rect = rect.try_cast::<usize>().unwrap();
        let stride = dst_size.width.try_into().unwrap();
        let start = rect.min.y * stride + rect.min.x;
        let prefix_per_stride = rect.width();
        let stride_count = rect.height();
        Some(Self {
            start,
            stride,
            prefix_per_stride,
            stride_count,
        })
    }

    pub fn fill<Blend>(&self, dst: &mut [u32], src: u32)
    where
        Blend: sw_composite::blend::Blend,
    {
        let dst = &mut dst[self.start..];
        for dst_chunk in dst.chunks_exact_mut(self.stride).take(self.stride_count) {
            for dst in dst_chunk.iter_mut().take(self.prefix_per_stride) {
                *dst = Blend::blend(src, *dst);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fill_strides::FillStrides;
    use euclid::default::{Box2D, Point2D, Size2D};

    #[test]
    fn small_over_big() {
        let dst_size = Size2D::new(300, 300);
        let src_size = Size2D::new(100, 100);
        let dst_pos = Point2D::new(100, 100);
        let dst_stride = dst_size.width as usize;
        let src_stride = src_size.width as usize;
        let f = FillStrides::init(dst_size, Box2D::from_origin_and_size(dst_pos, src_size)).unwrap();
        assert_eq!(
            f,
            FillStrides {
                start: dst_pos.x as usize + dst_stride * (dst_pos.y as usize),
                stride: dst_stride,
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
        let f = FillStrides::init(dst_size, Box2D::from_origin_and_size(dst_pos, src_size)).unwrap();
        assert_eq!(
            f,
            FillStrides {
                start: 0,
                stride: dst_stride,
                prefix_per_stride: dst_stride,
                stride_count: dst_size.height as usize,
            }
        );
    }
}
