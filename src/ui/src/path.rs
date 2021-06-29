use alloc::borrow::Cow;
use euclid::{Box2D, Point2D, Transform2D};
use num_traits::NumCast;
use raqote::{Path as RqPath, PathBuilder};

#[derive(Debug, Clone)]
pub enum Path<T, U> {
    Rect(Box2D<T, U>),
    Path(RqPath),
}

impl<T, U> Path<T, U>
where
    T: Copy + NumCast + PartialEq,
{
    pub fn as_int_rect(&self) -> Option<Box2D<i32, U>> {
        if let Self::Rect(r) = self {
            let int_r = r.cast();
            if &int_r.cast() == r {
                return Some(int_r);
            }
        }

        None
    }
}

impl<U> Path<f32, U> {
    pub fn transform<Dst>(self, transform: &Transform2D<f32, U, Dst>) -> Path<f32, Dst> {
        let p = match self {
            Self::Rect(r) => {
                if transform.m12 == 0.0 && transform.m21 == 0.0 {
                    let r = transform.outer_transformed_box(&r);
                    return Path::Rect(r);
                }

                let mut pb = PathBuilder::new();
                pb.rect(r.min.x, r.min.y, r.width(), r.height());
                pb.finish()
            }
            Self::Path(p) => p,
        };

        Path::Path(p.transform(&transform.to_untyped()))
    }

    pub fn to_rq_path(&self) -> Cow<RqPath> {
        match self {
            Self::Rect(r) => {
                let mut pb = PathBuilder::new();
                pb.rect(r.min.x, r.min.y, r.width(), r.height());
                Cow::Owned(pb.finish())
            }
            Self::Path(p) => Cow::Borrowed(p),
        }
    }

    pub fn contains(&self, tolerance: f32, p: Point2D<f32, U>) -> bool {
        match self {
            Self::Rect(r) => r.contains(p),
            Self::Path(path) => path.contains_point(tolerance, p.x, p.y),
        }
    }

    pub fn bounds(&self) -> Box2D<f32, U> {
        match self {
            Self::Rect(r) => *r,
            Self::Path(_) => todo!(),
        }
    }
}
