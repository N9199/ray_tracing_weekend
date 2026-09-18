#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, RangeInclusive, Sub},
};

use geometry::{
    aabbox::AABBox,
    bounded::Bounded,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

#[derive(Debug)]
pub struct Plane {
    point: Point3,
    normal: Vec3,
    mat_ptr: DynMaterial,
}

impl Plane {
    /// # Panics
    /// If `T` fails to be converted into `DynMaterial` this function will panic
    /// It'll panic if the normal is strictly down-facing (i.e. `normal.y` is negative and both `normal.x` and `normal.z` are zero)
    /// It'll panic if the normal is "zero", has `NaN` or `Inf` components.
    #[must_use]
    pub fn new<T>(point: Point3, normal: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        assert!(
            !(normal.y < 0. && normal.x.abs() < f64::EPSILON && normal.z.abs() < f64::EPSILON),
            "Normal is strictly down-facing"
        );
        assert!(
            !(normal.x.abs() < f64::EPSILON
                && normal.y.abs() < f64::EPSILON
                && normal.z.abs() < f64::EPSILON),
            "Normal is zero"
        );
        assert!(
            !(normal.x.is_nan() || normal.y.is_nan() || normal.z.is_nan()),
            "Normal has NaN components"
        );
        assert!(
            !(normal.x.is_infinite() || normal.y.is_infinite() || normal.z.is_infinite()),
            "Normal has Inf components"
        );
        Self {
            point,
            normal: normal.normalize(),
            mat_ptr: mat_ptr.try_into().unwrap(),
        }
    }

    #[must_use]
    #[inline(never)]
    pub fn get_plane_uv(&self, point: Point3) -> (f64, f64) {
        const V: Vec3 = Vec3::new(0., 1., 0.);
        let theta = f64::atan2(self.normal.cross(V).length(), self.normal.dot(V));
        if theta <= f64::EPSILON {
            return (point.x, point.z);
        }
        let k = self.normal.cross(V).normalize();
        let vec_to_rotate = point - self.point;
        let rotated_vec = vec_to_rotate
            .mul(theta.cos())
            .add(k.cross(vec_to_rotate).mul(theta.sin()))
            .add(k.mul(k.dot(vec_to_rotate)).mul((1.).sub(theta.cos())));
        (rotated_vec.x.fract(), rotated_vec.z.fract())
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static PLANE_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Plane {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let denom = r.get_direction().dot(self.normal);
        (denom < -f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.point).dot(self.normal).div(denom);
        let point = r.at(t);
        let (u, v) = self.get_plane_uv(point);
        assert!(
            u.is_finite() && v.is_finite(),
            "u = {u}, v = {v}, point = {point:?}"
        );
        (range.contains(&t)).then(|| {
            #[cfg(feature = "hit_counters")]
            PLANE_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
    }
}

impl Bounded for Plane {
    fn get_aabbox(&self) -> AABBox {
        let (x_min, x_max) =
            if self.normal.z.abs() < f64::EPSILON && self.normal.y.abs() < f64::EPSILON {
                (0., 0.)
            } else {
                (-f64::INFINITY, f64::INFINITY)
            };
        let (y_min, y_max) =
            if self.normal.x.abs() < f64::EPSILON && self.normal.z.abs() < f64::EPSILON {
                (0., 0.)
            } else {
                (-f64::INFINITY, f64::INFINITY)
            };
        let (z_min, z_max) =
            if self.normal.x.abs() < f64::EPSILON && self.normal.y.abs() < f64::EPSILON {
                (0., 0.)
            } else {
                (-f64::INFINITY, f64::INFINITY)
            };
        AABBox::new(
            Point3::new(x_min, y_min, z_min),
            Point3::new(x_max, y_max, z_max),
        )
    }

    fn get_surface_area(&self) -> f64 {
        f64::INFINITY
    }
}

impl BoundedHittable for Plane {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.hit(r, range).is_some()
    }
}

#[cfg(test)]
mod tests {
    use geometry::{
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance, assert_vec as assert_vec_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{entities::Plane, hittable::Hittable, material::INVISIBLE_PTR, ray::Ray};

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

    fn plane() -> Plane {
        Plane::new(Point3::zero(), Vec3::new(0., 1., 0.), INVISIBLE_PTR)
    }

    #[test]
    fn plane_hits_front_side_with_expected_point_normal_and_uv() {
        let plane = plane();
        let ray = Ray::new(Point3::new(1., 1., 2.), Vec3::new(0., -1., 0.));
        let hit = plane.hit(&ray, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 1.);
        assert_point(hit.get_p(), Point3::new(1., 0., 2.));
        assert_vec(hit.get_normal(), Vec3::new(0., 1., 0.));
        assert!(hit.is_front_face());
        assert_close(hit.get_u(), 1.);
        assert_close(hit.get_v(), 2.);
    }

    #[test]
    fn plane_respects_closed_range_and_one_sided_current_behavior() {
        let plane = plane();
        let front_ray = Ray::new(Point3::new(0., 1., 0.), Vec3::new(0., -1., 0.));
        assert_close(plane.hit(&front_ray, 1. ..=1.).unwrap().get_t(), 1.);
        assert!(plane.hit(&front_ray, 0. ..=0.999).is_none());
        assert!(plane.hit(&front_ray, 1.001..=2.).is_none());

        let back_ray = Ray::new(Point3::new(0., -1., 0.), Vec3::new(0., 1., 0.));
        assert!(plane.hit(&back_ray, 0. ..=f64::INFINITY).is_none());
    }

    #[test]
    fn plane_rejects_parallel_and_threshold_denominators() {
        let plane = plane();
        let parallel = Ray::new(Point3::new(0., 1., 0.), Vec3::new(1., 0., 0.));
        assert!(plane.hit(&parallel, 0. ..=f64::INFINITY).is_none());

        let threshold = Ray::new(Point3::new(0., 1., 0.), Vec3::new(0., -f64::EPSILON, 0.));
        assert!(plane.hit(&threshold, 0. ..=f64::INFINITY).is_none());
    }
}
