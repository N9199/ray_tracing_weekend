use std::{cmp::Ordering, ops::RangeInclusive};

#[cfg(feature = "euclid")]
use euclid::{Box3D, Point3D, UnknownUnit};

use crate::{
    aaplane::{AAPlane, Axis},
    bounded::Bounded,
};

#[cfg(feature = "euclid")]
pub type AABBox = Box3D<f64, UnknownUnit>;

#[cfg(feature = "euclid")]
fn has_valid_bounds(bounds: &AABBox) -> bool {
    bounds.min.x <= bounds.max.x && bounds.min.y <= bounds.max.y && bounds.min.z <= bounds.max.z
}

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
            Axis::X => self.min.x..=self.max.x,
            Axis::Y => self.min.y..=self.max.y,
            Axis::Z => self.min.z..=self.max.z,
        }
    }

    fn enclose<O: Bounded>(self, object: &O) -> Self {
        let other = object.get_aabbox();
        if !has_valid_bounds(&other) {
            return self;
        }
        if !has_valid_bounds(&self) {
            return other;
        }

        Self::new(
            Point3D::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Point3D::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }
}

#[cfg(all(test, feature = "euclid"))]
mod tests {
    use euclid::Point3D;

    use super::{AABBox, Box3DExt as _};

    #[test]
    fn union_of_planar_bounds_keeps_their_full_extent() {
        let horizontal_bounds = AABBox::new(Point3D::new(0., 0., 0.), Point3D::new(2., 3., 0.));
        let vertical_bounds = AABBox::new(Point3D::new(1., -1., -2.), Point3D::new(4., -1., 2.));

        let bounds = horizontal_bounds.enclose(&vertical_bounds);

        assert_eq!(bounds.min, Point3D::new(0., -1., -2.));
        assert_eq!(bounds.max, Point3D::new(4., 3., 2.));
    }

    #[test]
    fn enclose_includes_a_planar_bound_in_a_volume() {
        let volume = AABBox::new(Point3D::new(10., 10., 10.), Point3D::new(11., 11., 11.));
        let planar = AABBox::new(Point3D::new(-2., -3., 0.), Point3D::new(4., 5., 0.));

        let bounds = volume.enclose(&planar);

        assert_eq!(bounds.min, Point3D::new(-2., -3., 0.));
        assert_eq!(bounds.max, Point3D::new(11., 11., 11.));
    }

    #[test]
    fn inverted_bounds_remain_empty_for_enclosure() {
        let valid = AABBox::new(Point3D::new(0., 0., 0.), Point3D::new(2., 2., 2.));
        let inverted = AABBox::new(Point3D::new(3., 0., 0.), Point3D::new(2., 2., 2.));

        let bounds = inverted.enclose(&valid);

        assert_eq!(bounds.min, valid.min);
        assert_eq!(bounds.max, valid.max);
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
                Some(aabbox) => Some(aabbox.enclose(&item)),
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

#[cfg(test)]
mod geometry_tests {
    use std::cmp::Ordering;

    #[cfg(feature = "euclid")]
    use crate::aabbox::Box3DExt as _;
    use crate::{
        aabbox::AABBox,
        aaplane::{AAPlane, Axis},
        bounded::Bounded,
        test_utils::{assert_close, assert_point},
        vec3::{Point3, Vec3},
    };

    const TOLERANCE: f64 = 1e-12;

    fn bounds(min: [f64; 3], max: [f64; 3]) -> AABBox {
        AABBox::new(Point3::from(min), Point3::from(max))
    }

    fn assert_bounds(actual: &AABBox, min: [f64; 3], max: [f64; 3]) {
        let actual_min = Point3::new(
            *actual.axis(Axis::X).start(),
            *actual.axis(Axis::Y).start(),
            *actual.axis(Axis::Z).start(),
        );
        let actual_max = Point3::new(
            *actual.axis(Axis::X).end(),
            *actual.axis(Axis::Y).end(),
            *actual.axis(Axis::Z).end(),
        );
        assert_point(actual_min, Point3::from(min), TOLERANCE);
        assert_point(actual_max, Point3::from(max), TOLERANCE);
    }

    #[test]
    fn coordinates_corners_and_axis_ranges_are_correct() {
        let box3 = bounds([-1., 2., 0.], [3., 5., 7.]);
        assert_eq!(*box3.axis(Axis::X).start(), -1.);
        assert_eq!(*box3.axis(Axis::X).end(), 3.);
        assert_eq!(*box3.axis(Axis::Y).start(), 2.);
        assert_eq!(*box3.axis(Axis::Y).end(), 5.);
        assert_eq!(*box3.axis(Axis::Z).start(), 0.);
        assert_eq!(*box3.axis(Axis::Z).end(), 7.);

        let mut actual: Vec<[f64; 3]> = box3
            .get_points()
            .into_iter()
            .map(Point3::to_array)
            .collect::<Vec<_>>();
        let mut expected: Vec<[f64; 3]> = Vec::new();
        for x in [-1., 3.] {
            for y in [2., 5.] {
                for z in [0., 7.] {
                    expected.push([x, y, z]);
                }
            }
        }
        actual.sort_by(|a, b| {
            a[0].total_cmp(&b[0])
                .then(a[1].total_cmp(&b[1]))
                .then(a[2].total_cmp(&b[2]))
        });
        expected.sort_by(|a, b| {
            a[0].total_cmp(&b[0])
                .then(a[1].total_cmp(&b[1]))
                .then(a[2].total_cmp(&b[2]))
        });
        assert_eq!(actual, expected);
    }

    #[test]
    fn enclosure_contains_inside_face_and_outside_points_and_is_commutative() {
        let original = bounds([-1., 2., 0.], [3., 5., 7.]);
        for point in [
            Vec3::new(0., 3., 2.),
            Vec3::new(-1., 4., 1.),
            Vec3::new(4., 1., 8.),
        ] {
            let enclosed = original.enclose(&point);
            assert!(enclosed.axis(Axis::X).start() <= &point.x);
            assert!(enclosed.axis(Axis::X).end() >= &point.x);
            assert!(enclosed.axis(Axis::Y).start() <= &point.y);
            assert!(enclosed.axis(Axis::Y).end() >= &point.y);
            assert!(enclosed.axis(Axis::Z).start() <= &point.z);
            assert!(enclosed.axis(Axis::Z).end() >= &point.z);
            assert!(enclosed.axis(Axis::X).start() <= original.axis(Axis::X).start());
            assert!(enclosed.axis(Axis::X).end() >= original.axis(Axis::X).end());
        }
        assert_bounds(
            &original.enclose(&Vec3::new(4., 1., 8.)),
            [-1., 1., 0.],
            [4., 5., 8.],
        );

        let a = bounds([-4., -3., -2.], [-2., -1., 0.]);
        let b = bounds([1., 2., 3.], [4., 5., 6.]);
        let enclosed_ab = a.enclose(&b);
        let enclosed_ba = b.enclose(&a);
        assert_bounds(&enclosed_ab, [-4., -3., -2.], [4., 5., 6.]);
        assert_bounds(&enclosed_ba, [-4., -3., -2.], [4., 5., 6.]);
        assert_bounds(&enclosed_ab.enclose(&a), [-4., -3., -2.], [4., 5., 6.]);
    }

    #[test]
    fn strict_left_right_and_minimum_axis_comparisons_are_correct() {
        let box3 = bounds([1., 1., 1.], [3., 3., 3.]);
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            assert!(box3.left_of(AAPlane { coord: 4., axis }));
            assert!(box3.right_of(AAPlane { coord: 0., axis }));
            for coordinate in [1., 2., 3.] {
                assert!(!box3.left_of(AAPlane {
                    coord: coordinate,
                    axis
                }));
                assert!(!box3.right_of(AAPlane {
                    coord: coordinate,
                    axis
                }));
            }
        }

        let x_early = bounds([0., 100., 100.], [1., 101., 101.]);
        let x_late = bounds([2., 0., 0.], [3., 1., 1.]);
        assert_eq!(x_early.compare_by_axis(&x_late, Axis::X), Ordering::Less);
        assert_eq!(x_early.compare_by_axis(&x_late, Axis::Y), Ordering::Greater);
        assert_eq!(x_early.compare_by_axis(&x_late, Axis::Z), Ordering::Greater);
        let same_starts = bounds([0., 100., 100.], [8., 101., 101.]);
        assert_eq!(
            x_early.compare_by_axis(&same_starts, Axis::X),
            Ordering::Equal
        );
    }

    #[test]
    fn surface_area_matches_box_dimensions_and_bounded_behavior() {
        let box3 = bounds([0., 0., 0.], [2., 3., 4.]);
        assert_close(box3.get_surface_area(), 52., TOLERANCE);
        assert_close(
            bounds([0., 0., 0.], [2., 3., 0.]).get_surface_area(),
            12.,
            TOLERANCE,
        );
        assert_close(
            bounds([0., 0., 0.], [2., 0., 0.]).get_surface_area(),
            0.,
            TOLERANCE,
        );
        assert_close(
            bounds([0., 0., 0.], [0., 0., 0.]).get_surface_area(),
            0.,
            TOLERANCE,
        );
        let translated = bounds([5., -2., 9.], [7., 1., 13.]);
        assert_close(
            translated.get_surface_area(),
            box3.get_surface_area(),
            TOLERANCE,
        );
        assert_close(
            box3.enclose(&bounds([-1., 0., 0.], [2., 5., 4.]))
                .get_surface_area(),
            94.,
            TOLERANCE,
        );

        let point = Vec3::new(2., -3., 4.);
        let point_bounds = point.get_aabbox();
        assert!(point_bounds.axis(Axis::X).contains(&point.x));
        assert!(point_bounds.axis(Axis::Y).contains(&point.y));
        assert!(point_bounds.axis(Axis::Z).contains(&point.z));
        assert_close(point.get_surface_area(), 0., TOLERANCE);
        let copied_bounds = box3.get_aabbox();
        assert_bounds(&copied_bounds, [0., 0., 0.], [2., 3., 4.]);
    }

    #[cfg(not(feature = "euclid"))]
    #[test]
    fn custom_backend_preserves_exact_point_construction_and_pads_enclosures() {
        let point = Point3::new(1., 2., 3.);
        assert_bounds(&AABBox::from(point), [1., 2., 3.], [1., 2., 3.]);
        assert_bounds(&AABBox::from_points([point]), [1., 2., 3.], [1., 2., 3.]);
        assert_bounds(
            &AABBox::from_points([Point3::new(3., 2., 1.), Point3::new(-1., 4., 5.)]),
            [-1., 2., 1.],
            [3., 4., 5.],
        );

        let padded = AABBox::from(point).enclose(&point);
        assert_bounds(
            &padded,
            [1. - 0.0001, 2. - 0.0001, 3. - 0.0001],
            [1. + 0.0001, 2. + 0.0001, 3. + 0.0001],
        );
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            assert_close(axis_width(&padded, axis), 0.0002, 1e-12);
        }
    }

    #[cfg(feature = "euclid")]
    #[test]
    fn euclid_backend_keeps_degenerate_enclosures_unpadded() {
        let point = Vec3::new(1., 2., 3.);
        let point_box = AABBox::new(Point3::from([1., 2., 3.]), Point3::from([1., 2., 3.]));
        let enclosed = point_box.enclose(&point);
        assert_bounds(&enclosed, [1., 2., 3.], [1., 2., 3.]);
        assert_close(enclosed.get_surface_area(), 0., TOLERANCE);
    }

    #[cfg(not(feature = "euclid"))]
    fn axis_width(box3: &AABBox, axis: Axis) -> f64 {
        box3.axis(axis).end() - box3.axis(axis).start()
    }
}
