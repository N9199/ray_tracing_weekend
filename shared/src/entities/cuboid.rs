use std::{fmt::Debug, ops::RangeInclusive};

#[cfg(feature = "euclid")]
use geometry::aabbox::Box3DExt as _;

use geometry::{
    aabbox::AABBox,
    aaplane::get_axis,
    bounded::Bounded,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

use super::Quad;

#[derive(Debug, Clone)]
pub struct Cuboid {
    quads: [Quad; 6],
}

impl Cuboid {
    /// # Panics
    /// If `T` fails to be converted into `DynMaterial` this function will panic
    #[must_use]
    pub fn new<T>(p: Point3, q: Point3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabox = AABBox::from_points([p, q]);
        let min_p = Point3::from(get_axis().map(|axis| *aabox.axis(axis).start()));
        let max_p = Point3::from(get_axis().map(|axis| *aabox.axis(axis).end()));
        let delta = max_p - min_p;
        let dx = Vec3::new(delta.x, 0., 0.);
        let dy = Vec3::new(0., delta.y, 0.);
        let dz = Vec3::new(0., 0., delta.z);
        let quads = [
            Quad::new(min_p, dx, dy, mat_ptr.clone()),
            Quad::new(min_p, dy, dz, mat_ptr.clone()),
            Quad::new(min_p, dx, dz, mat_ptr.clone()),
            Quad::new(max_p, -dx, -dy, mat_ptr.clone()),
            Quad::new(max_p, -dy, -dz, mat_ptr.clone()),
            Quad::new(max_p, -dx, -dz, mat_ptr.clone()),
        ];
        Self { quads }
    }
}

impl Hittable for Cuboid {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        self.quads
            .iter()
            .filter_map(|q| q.hit(r, range.clone()))
            .min_by(|hit1, hit2| hit1.get_t().total_cmp(&hit2.get_t()))
    }
}

impl Bounded for Cuboid {
    fn get_aabbox(&self) -> AABBox {
        self.quads
            .iter()
            .fold(None, |accum, v| {
                accum
                    .map_or_else(|| v.get_aabbox(), |b: AABBox| b.enclose(v))
                    .into()
            })
            .unwrap()
    }

    fn get_surface_area(&self) -> f64 {
        self.quads.iter().map(Bounded::get_surface_area).sum()
    }
}

impl BoundedHittable for Cuboid {}

#[cfg(test)]
mod tests {
    #[cfg(feature = "euclid")]
    use geometry::aabbox::Box3DExt as _;
    use geometry::{
        bounded::Bounded,
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{entities::Cuboid, hittable::Hittable, material::INVISIBLE_PTR, ray::Ray};

    const TOLERANCE: f64 = 1e-10;

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_point(actual: Point3, expected: Point3) {
        assert_point_with_tolerance(actual, expected, TOLERANCE);
    }

    fn cuboid(p: Point3, q: Point3) -> Cuboid {
        Cuboid::new(p, q, INVISIBLE_PTR)
    }

    #[test]
    fn cuboid_returns_nearest_face_hit_and_inside_exit() {
        let cuboid = cuboid(Point3::zero(), Point3::new(2., 2., 2.));
        let outside = Ray::new(Point3::new(0.5, 1., -2.), Vec3::new(0., 0., 1.));
        let hit = cuboid.hit(&outside, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 2.);
        assert_point(hit.get_p(), Point3::new(0.5, 1., 0.));
        assert_close(hit.get_u(), 0.25);
        assert_close(hit.get_v(), 0.5);
        assert!(!hit.is_front_face());
        assert!(hit.get_normal().dot(outside.get_direction()) <= 0.);

        let inside = Ray::new(Point3::new(1., 1., 1.), Vec3::new(0., 0., 1.));
        let hit = cuboid.hit(&inside, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 1.);
        assert_point(hit.get_p(), Point3::new(1., 1., 2.));
        assert!(hit.get_normal().dot(inside.get_direction()) <= 0.);
    }

    #[test]
    fn cuboid_normalizes_reversed_corners_and_has_expected_bounds_and_area() {
        let cuboid = cuboid(Point3::new(2., 2., 2.), Point3::zero());
        let ray = Ray::new(Point3::new(1., 1., -2.), Vec3::new(0., 0., 1.));
        let hit = cuboid.hit(&ray, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 2.);
        assert_point(hit.get_p(), Point3::new(1., 1., 0.));

        let bounds = cuboid.get_aabbox();
        for axis in [
            geometry::aaplane::Axis::X,
            geometry::aaplane::Axis::Y,
            geometry::aaplane::Axis::Z,
        ] {
            assert!(*bounds.axis(axis).start() <= 0.);
            assert!(*bounds.axis(axis).start() >= -0.000_11);
            assert!(*bounds.axis(axis).end() >= 2.);
            assert!(*bounds.axis(axis).end() <= 2.000_11);
        }
        assert_close(cuboid.get_surface_area(), 24.);
    }
}
