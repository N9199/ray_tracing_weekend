#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};

use std::fmt::Debug;
use std::ops::{Div, RangeInclusive, Sub};

use geometry::bounded::Bounded;
use geometry::{
    aabbox::AABBox,
    vec3::{Point3, Vec3},
};
use rand::distributions::Open01;
use rand::Rng as _;

use crate::hittable::{BoundedHittable, HitRecord, Hittable};
use crate::material::DynMaterial;
use crate::ray::Ray;
use crate::utils::random_utils::UNIT;

#[cfg(feature = "hit_counters")]
pub(crate) static TRIANGLES_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct Triangle {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    mat_ptr: DynMaterial,
    aabbox: AABBox,
    area: f64,
}

impl Triangle {
    /// # Panics
    /// If `T` fails to be converted into `DynMaterial` this function will panic
    #[must_use]
    pub fn new<T>(q: Point3, u: Vec3, v: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabbox = AABBox::from_points([(q + (u + v) * 0.5), q, q + v, q + u]);
        let normal = u.cross(v);
        let w = normal.div(normal.square_length());
        let area = normal.length() / 2.;
        let normal = normal.unit_vec();
        Self {
            q,
            u,
            v,
            w,
            normal,
            mat_ptr,
            aabbox,
            area,
        }
    }

    fn get_triangle_uv(&self, point: Point3) -> (f64, f64) {
        (
            point.sub(self.q).cross(self.v).dot(self.w),
            self.u.cross(point.sub(self.q)).dot(self.w),
        )
    }
}

impl Bounded for Triangle {
    fn get_aabbox(&self) -> AABBox {
        self.aabbox
    }

    fn get_surface_area(&self) -> f64 {
        self.area
    }
}

impl Hittable for Triangle {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let denom = r.get_direction().dot(self.normal);
        (denom.abs() > f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.q).dot(self.normal).div(denom);
        (range.contains(&t)).then_some(())?;
        let point = r.at(t);
        let (u, v) = self.get_triangle_uv(point);
        // dbg!(UNIT.contains(&u), UNIT.contains(&v), u, v, point);
        (UNIT.contains(&(u + v)) && u >= 0. && v >= 0.).then(|| {
            #[cfg(feature = "hit_counters")]
            TRIANGLES_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            // dbg!(self.mat_ptr.as_ref());
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self.hit(&Ray::new(origin, direction), (0.)..=f64::INFINITY) {
            Some(record) => {
                let distance_squared = record.get_t() * record.get_t() * direction.square_length();
                let cosine = direction
                    .dot(record.get_normal())
                    .div(direction.length())
                    .abs();
                distance_squared / (cosine * self.area)
            }
            None => 0.,
        }
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        let mut r1 = rng.sample::<f64, _>(Open01);
        let mut r2 = rng.sample::<f64, _>(Open01);
        if r1 + r2 > 1. {
            r1 = 1. - r1;
            r2 = 1. - r2;
        }
        let p = self.q + self.u * r1 + self.v * r2;
        p - origin
    }
}

impl BoundedHittable for Triangle {}

#[cfg(test)]
mod tests {
    use geometry::{
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance, assert_vec as assert_vec_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{entities::Triangle, hittable::Hittable, material::INVISIBLE_PTR, ray::Ray};

    const TOLERANCE: f64 = 1e-10;

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_vec(actual: Vec3, expected: Vec3) {
        assert_vec_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_point(actual: Point3, expected: Point3) {
        assert_point_with_tolerance(actual, expected, TOLERANCE);
    }

    fn triangle() -> Triangle {
        Triangle::new(
            Point3::zero(),
            Vec3::new(2., 0., 0.),
            Vec3::new(0., 2., 0.),
            INVISIBLE_PTR,
        )
    }

    #[test]
    fn triangle_hits_interior_with_expected_uv_and_two_sided_normal() {
        let triangle = triangle();
        let ray = Ray::new(Point3::new(0.5, 0.5, -2.), Vec3::new(0., 0., 1.));
        let hit = triangle.hit(&ray, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 2.);
        assert_point(hit.get_p(), Point3::new(0.5, 0.5, 0.));
        assert_close(hit.get_u(), 0.25);
        assert_close(hit.get_v(), 0.25);
        assert!(!hit.is_front_face());
        assert_vec(hit.get_normal(), Vec3::new(0., 0., -1.));

        let reverse = Ray::new(Point3::new(0.5, 0.5, 2.), Vec3::new(0., 0., -1.));
        let reverse_hit = triangle.hit(&reverse, 2. ..=2.).unwrap();
        assert!(reverse_hit.is_front_face());
        assert_vec(reverse_hit.get_normal(), Vec3::new(0., 0., 1.));
    }

    #[test]
    fn triangle_includes_vertices_edges_and_diagonal_but_rejects_outside_points() {
        let triangle = triangle();
        for (x, y, u, v) in [
            (0., 0., 0., 0.),
            (2., 0., 1., 0.),
            (0., 2., 0., 1.),
            (1., 1., 0.5, 0.5),
            (1., 0., 0.5, 0.),
            (0., 1., 0., 0.5),
        ] {
            let ray = Ray::new(Point3::new(x, y, -1.), Vec3::new(0., 0., 1.));
            let hit = triangle.hit(&ray, 1. ..=1.).unwrap();
            assert_close(hit.get_u(), u);
            assert_close(hit.get_v(), v);
        }

        let beyond_diagonal = Ray::new(Point3::new(1.5, 1.5, -1.), Vec3::new(0., 0., 1.));
        assert!(triangle.hit(&beyond_diagonal, 0. ..=2.).is_none());
        let negative_u = Ray::new(Point3::new(-0.001, 0.5, -1.), Vec3::new(0., 0., 1.));
        assert!(triangle.hit(&negative_u, 0. ..=2.).is_none());
        let negative_v = Ray::new(Point3::new(0.5, -0.001, -1.), Vec3::new(0., 0., 1.));
        assert!(triangle.hit(&negative_v, 0. ..=2.).is_none());
    }

    #[test]
    fn triangle_rejects_parallel_rays_and_respects_range() {
        let triangle = triangle();
        let ray = Ray::new(Point3::new(0.5, 0.5, -2.), Vec3::new(0., 0., 1.));
        assert!(triangle.hit(&ray, 2. ..=2.).is_some());
        assert!(triangle.hit(&ray, 0. ..=1.999).is_none());

        let parallel = Ray::new(Point3::new(0.5, 0.5, 1.), Vec3::new(1., 0., 0.));
        assert!(triangle.hit(&parallel, 0. ..=f64::INFINITY).is_none());
        let threshold = Ray::new(Point3::new(0.5, 0.5, 1.), Vec3::new(0., 0., f64::EPSILON));
        assert!(triangle.hit(&threshold, 0. ..=f64::INFINITY).is_none());
    }
}
