use std::{cmp::Ordering, ops::RangeInclusive};

#[cfg(feature = "euclid")]
use euclid::{Box3D, Point3D, UnknownUnit};

use crate::{
    aaplane::{AAPlane, Axis},
    bounded::Bounded,
};

#[cfg(feature = "euclid")]
pub type AABBox = Box3D<f64, UnknownUnit>;

#[cfg(not(feature = "euclid"))]
pub use inner::AABBox;

pub trait Box3DExt {
    type Point;
    fn get_points(&self) -> [Self::Point; 8];

    fn axis(&self, axis: Axis) -> RangeInclusive<f64>;

    #[must_use]
    fn enclose<T: Bounded>(self, object: &T) -> Self;

    fn left_of(&self, plane: AAPlane) -> bool {
        self.axis(plane.axis as _).end() < &plane.coord
    }

    fn right_of(&self, plane: AAPlane) -> bool {
        self.axis(plane.axis as _).start() > &plane.coord
    }

    fn compare_by_axis(&self, other: &Self, axis: Axis) -> Ordering {
        self.axis(axis).start().total_cmp(other.axis(axis).start())
    }
}

#[cfg(feature = "euclid")]
impl Bounded for Box3D<f64, UnknownUnit> {
    fn get_aabbox(&self) -> AABBox {
        *self
    }

    fn get_surface_area(&self) -> f64 {
        2. * (self.xy_area() + self.xz_area() + self.yz_area())
    }
}
#[cfg(feature = "euclid")]
impl Box3DExt for Box3D<f64, UnknownUnit> {
    type Point = Point3D<f64, UnknownUnit>;

    fn get_points(&self) -> [Self::Point; 8] {
        [
            Point3D::new(self.min.x, self.min.y, self.min.z),
            Point3D::new(self.min.x, self.max.y, self.min.z),
            Point3D::new(self.min.x, self.min.y, self.max.z),
            Point3D::new(self.min.x, self.max.y, self.max.z),
            Point3D::new(self.max.x, self.min.y, self.min.z),
            Point3D::new(self.max.x, self.max.y, self.min.z),
            Point3D::new(self.max.x, self.min.y, self.max.z),
            Point3D::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    fn axis(&self, axis: Axis) -> RangeInclusive<f64> {
        match axis {
            Axis::Z => self.min.x..=self.max.x,
            Axis::X => self.min.y..=self.max.y,
            Axis::Y => self.min.z..=self.max.z,
        }
    }

    fn enclose<O: Bounded>(self, object: &O) -> Self {
        self.union(&object.get_aabbox())
    }
}

#[cfg(not(feature = "euclid"))]
mod inner {
    use std::{borrow::Borrow, cmp::Ordering, ops::RangeInclusive};

    use crate::{bounded::Bounded, vec3::Point3};

    use super::super::aaplane::{AAPlane, Axis};
    #[derive(Debug, Clone, Copy)]
    pub struct AABBox {
        min: Point3,
        max: Point3,
    }

    impl FromIterator<Point3> for Option<AABBox> {
        fn from_iter<T: IntoIterator<Item = Point3>>(iter: T) -> Self {
            iter.into_iter().fold(None, |accum, item| match accum {
                Some(aabox) => Some(aabox.enclose(&item)),
                None => Some(item.into()),
            })
        }
    }

    impl AABBox {
        #[must_use]
        pub const fn new(min: Point3, max: Point3) -> Self {
            Self { min, max }
        }

        #[must_use]
        pub const fn zero() -> Self {
            let min = Point3::zero();
            let max = Point3::zero();
            Self { min, max }
        }

        #[must_use]
        pub const fn get_points(self) -> [Point3; 8] {
            [
                Point3::new(self.min.x, self.min.y, self.min.z),
                Point3::new(self.min.x, self.max.y, self.min.z),
                Point3::new(self.min.x, self.min.y, self.max.z),
                Point3::new(self.min.x, self.max.y, self.max.z),
                Point3::new(self.max.x, self.min.y, self.min.z),
                Point3::new(self.max.x, self.max.y, self.min.z),
                Point3::new(self.max.x, self.min.y, self.max.z),
                Point3::new(self.max.x, self.max.y, self.max.z),
            ]
        }

        // TODO: When float operations are const, constify this function
        fn pad_to_minimum(mut self) -> Self {
            const DELTA: f64 = 0.0001;

            let dx = self.axis(Axis::X).end() - self.axis(Axis::X).start();
            let dy = self.axis(Axis::Y).end() - self.axis(Axis::Y).start();
            let dz = self.axis(Axis::Z).end() - self.axis(Axis::Z).start();

            if dx < DELTA {
                self.min.x -= DELTA;
                self.max.x += DELTA;
            }
            if dy < DELTA {
                self.min.y -= DELTA;
                self.max.y += DELTA;
            }
            if dz < DELTA {
                self.min.z -= DELTA;
                self.max.z += DELTA;
            }
            self
        }

        #[inline]
        #[must_use]
        pub const fn axis(&self, axis: Axis) -> RangeInclusive<f64> {
            match axis {
                Axis::X => self.min.x..=self.max.x,
                Axis::Y => self.min.y..=self.max.y,
                Axis::Z => self.min.z..=self.max.z,
            }
        }

        #[must_use]
        pub fn enclose<T: Bounded>(mut self, object: &T) -> Self {
            self.enclose_aabbox(object.get_aabbox());
            self
        }

        fn enclose_aabbox(&mut self, aabbox: AABBox) {
            self.min.x = self.min.x.min(aabbox.min.x);
            self.max.x = self.max.x.max(aabbox.max.x);
            self.min.y = self.min.y.min(aabbox.min.y);
            self.max.y = self.max.y.max(aabbox.max.y);
            self.min.z = self.min.z.min(aabbox.min.z);
            self.max.z = self.max.z.max(aabbox.max.z);
            *self = self.pad_to_minimum();
        }

        #[must_use]
        pub fn left_of(&self, plane: AAPlane) -> bool {
            self.axis(plane.axis as _).end() < &plane.coord
        }

        #[must_use]
        pub fn right_of(&self, plane: AAPlane) -> bool {
            self.axis(plane.axis as _).start() > &plane.coord
        }

        #[must_use]
        pub fn compare_by_axis(&self, other: &Self, axis: Axis) -> Ordering {
            self.axis(axis).start().total_cmp(other.axis(axis).start())
        }

        /// # Panics
        /// If the iterator is empty this will panic
        pub fn from_points<I>(points: I) -> Self
        where
            I: IntoIterator,
            I::Item: Borrow<Point3>,
        {
            let mut iter = points.into_iter();
            let first = iter.next().unwrap();
            let mut out = Self::from(*first.borrow());
            for point in iter {
                out = out.enclose(point.borrow());
            }
            out
        }
    }
    impl Bounded for AABBox {
        #[inline]
        fn get_aabbox(&self) -> AABBox {
            *self
        }

        fn get_surface_area(&self) -> f64 {
            let dx = self.max.x - self.min.x;
            let dy = self.max.y - self.min.y;
            let dz = self.max.z - self.min.z;
            2. * (dx * dy + dx * dz + dy * dz)
        }
    }

    impl From<Point3> for AABBox {
        fn from(value: Point3) -> Self {
            Self::new(value, value)
        }
    }
}
