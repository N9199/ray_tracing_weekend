use core::ops::RangeInclusive;
use std::fmt::Debug;

use crate::{material::Material, ray::Ray};

use geometry::{
    bounded::Bounded,
    vec3::{Point3, Vec3},
};

pub use aabox_extend::AABoxHit;

#[cfg(feature = "hit_counters")]
pub(crate) use aabox_extend::AABOX_HIT_COUNTER;

mod aabox_extend {
    use std::ops::RangeInclusive;
    #[cfg(feature = "hit_counters")]
    use std::sync::atomic::{self, AtomicU32};

    #[cfg(feature = "euclid")]
    use geometry::aabbox::Box3DExt as _;
    use geometry::{aabbox::AABBox, aaplane, bounded::Bounded};

    use crate::ray::Ray;

    #[cfg(feature = "hit_counters")]
    pub(crate) static AABOX_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

    pub trait AABoxHit: Bounded {
        fn is_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
            self.hit(r, range).is_some()
        }

        fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<f64>;
    }

    impl AABoxHit for AABBox {
        fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<f64> {
            let (x_min, x_max) = self.axis(aaplane::Axis::X).into_inner();
            let (y_min, y_max) = self.axis(aaplane::Axis::Y).into_inner();
            let (z_min, z_max) = self.axis(aaplane::Axis::Z).into_inner();
            // // dbg!(r);
            let x_tmin = (x_min - r.get_origin().x) / r.get_direction().x;
            let x_tmax = (x_max - r.get_origin().x) / r.get_direction().x;
            let (x_tmin, x_tmax) = if r.get_direction().x.is_sign_negative() {
                (x_tmax, x_tmin)
            } else {
                (x_tmin, x_tmax)
            };
            let (tmin, tmax) = (x_tmin, x_tmax);
            let y_tmin = (y_min - r.get_origin().y) / r.get_direction().y;
            let y_tmax = (y_max - r.get_origin().y) / r.get_direction().y;
            let (y_tmin, y_tmax) = if r.get_direction().y.is_sign_negative() {
                (y_tmax, y_tmin)
            } else {
                (y_tmin, y_tmax)
            };
            // // dbg!(tmax, tmin, y_tmin, y_tmax);
            if tmax < y_tmin || tmin > y_tmax {
                return None;
            }
            let (tmin, tmax) = (tmin.max(y_tmin), tmax.min(y_tmax));
            let z_tmin = (z_min - r.get_origin().z) / r.get_direction().z;
            let z_tmax = (z_max - r.get_origin().z) / r.get_direction().z;
            let (z_tmin, z_tmax) = if r.get_direction().z.is_sign_negative() {
                (z_tmax, z_tmin)
            } else {
                (z_tmin, z_tmax)
            };
            // // dbg!(tmax, tmin, z_tmin, z_tmax);
            if tmax < z_tmin || tmin > z_tmax {
                return None;
            }
            let (tmin, tmax) = (tmin.max(z_tmin), tmax.min(z_tmax));
            // TODO check if this are all the cases
            let out = range.start().max(tmin) <= range.end().min(tmax);

            #[cfg(feature = "hit_counters")]
            if out {
                // dbg!("AABox Hit");
                AABOX_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            }
            // dbg!(*self, r, tmin, tmax, &range, out);
            out.then_some(range.start().max(tmin))
        }
    }
}

pub struct HitRecord<'a> {
    p: Point3,
    normal: Vec3,
    t: f64,
    u: f64,
    v: f64,
    front_face: bool,
    mat_ptr: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    #[inline]
    pub fn new(
        r: &Ray,
        t: f64,
        outward_normal: Vec3,
        u: f64,
        v: f64,
        mat_ptr: &'a dyn Material,
    ) -> Self {
        // assert!(u.is_finite(), "{}", u);
        // assert!(v.is_finite(), "{}", v);
        // assert!(t.is_finite(), "{}", t);
        let p = r.at(t);
        let front_face = r.get_direction().dot(outward_normal) < 0.;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            p,
            normal,
            t,
            front_face,
            mat_ptr,
            u,
            v,
        }
    }

    #[inline]
    #[must_use]
    pub const fn get_u(&self) -> f64 {
        self.u
    }

    #[inline]
    #[must_use]
    pub const fn get_v(&self) -> f64 {
        self.v
    }

    #[inline]
    #[must_use]
    pub const fn get_p(&self) -> Point3 {
        self.p
    }

    #[inline]
    #[must_use]
    pub const fn get_normal(&self) -> Vec3 {
        self.normal
    }

    #[inline]
    #[must_use]
    pub const fn get_t(&self) -> f64 {
        self.t
    }

    #[inline]
    #[must_use]
    pub const fn is_front_face(&self) -> bool {
        self.front_face
    }

    #[inline]
    #[must_use]
    pub const fn get_material(&self) -> &dyn Material {
        self.mat_ptr
    }

    #[inline]
    pub(crate) const fn get_mut_p(&mut self) -> &mut Point3 {
        &mut self.p
    }

    #[inline]
    pub(crate) const fn get_mut_normal(&mut self) -> &mut Vec3 {
        &mut self.normal
    }
}

pub trait Hittable: Sync + Send + Debug {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>>;

    fn pdf_value(&self, _origin: Point3, _direction: Vec3) -> f64 {
        0.
    }

    fn random(&self, _origin: Point3, _rng: &mut dyn rand::RngCore) -> Vec3 {
        Vec3::from([1., 0., 0.])
    }
}

pub trait BoundedHittable: Hittable + Bounded + Debug {
    #[inline]
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.get_aabbox().is_hit(r, range)
    }

    #[inline]
    fn bounded_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        self.get_aabbox()
            .is_hit(r, range.clone())
            .then(|| self.hit(r, range))
            .flatten()
    }
}

impl<T> Hittable for &[T]
where
    T: BoundedHittable,
{
    #[inline]
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        self.iter()
            .filter_map(|obj| obj.bounded_hit(r, start..=end))
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }
}

#[cfg(test)]
mod tests {
    use geometry::{
        aabbox::AABBox,
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance, assert_vec as assert_vec_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{
        entities::Sphere,
        hittable::{AABoxHit as _, BoundedHittable, HitRecord},
        material::INVISIBLE_PTR,
        ray::Ray,
    };

    const TOLERANCE: f64 = 1e-10;

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_point(actual: Point3, expected: Point3) {
        assert_point_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_vec(actual: Vec3, expected: Vec3) {
        assert_vec_with_tolerance(actual, expected, TOLERANCE);
    }

    fn box_fixture() -> AABBox {
        AABBox::new(Point3::new(0., 0., 0.), Point3::new(1., 1., 1.))
    }

    #[test]
    fn aabb_hits_misses_and_swaps_negative_axis_intervals() {
        let aabb = box_fixture();
        let through_x = Ray::new(Point3::new(-1., 0.5, 0.5), Vec3::new(1., 0., 0.));
        assert_close(aabb.hit(&through_x, 0. ..=f64::INFINITY).unwrap(), 1.);
        assert!(aabb.is_hit(&through_x, 0. ..=f64::INFINITY));

        let outside_y = Ray::new(Point3::new(-1., 2., 0.5), Vec3::new(1., 0., 0.));
        assert_eq!(aabb.hit(&outside_y, 0. ..=f64::INFINITY), None);

        let reverse_x = Ray::new(Point3::new(2., 0.5, 0.5), Vec3::new(-1., 0., 0.));
        assert_close(aabb.hit(&reverse_x, 0. ..=f64::INFINITY).unwrap(), 1.);

        let reverse_y = Ray::new(Point3::new(0.5, 2., 0.5), Vec3::new(0., -1., 0.));
        assert_close(aabb.hit(&reverse_y, 0. ..=f64::INFINITY).unwrap(), 1.);
    }

    #[test]
    fn aabb_clips_to_inclusive_ray_parameter_range() {
        let aabb = box_fixture();
        let ray = Ray::new(Point3::new(-1., 0.5, 0.5), Vec3::new(1., 0., 0.));
        assert_close(aabb.hit(&ray, 0. ..=f64::INFINITY).unwrap(), 1.);
        assert_close(aabb.hit(&ray, 1.5..=3.).unwrap(), 1.5);
        assert_eq!(aabb.hit(&ray, 0. ..=0.999), None);
        assert_close(aabb.hit(&ray, 1. ..=1.).unwrap(), 1.);
        assert_eq!(aabb.hit(&ray, 0. ..=0.999_999), None);
    }

    #[test]
    fn aabb_accepts_slab_tangency() {
        let aabb = box_fixture();
        let ray = Ray::new(Point3::new(-1., 0., 0.5), Vec3::new(1., 1., 0.));
        assert_close(aabb.hit(&ray, 0. ..=f64::INFINITY).unwrap(), 1.);
        assert_close(aabb.hit(&ray, 1. ..=1.).unwrap(), 1.);
    }

    #[test]
    fn hit_record_orients_normal_and_preserves_ray_parameter_and_uvs() {
        let ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(0., 0., -2.));
        let front = HitRecord::new(&ray, 0.5, Vec3::new(0., 0., 1.), 0.25, 0.75, INVISIBLE_PTR);
        assert_point(front.get_p(), Point3::new(1., 2., 2.));
        assert_close(front.get_t(), 0.5);
        assert_close(front.get_u(), 0.25);
        assert_close(front.get_v(), 0.75);
        assert!(front.is_front_face());
        assert_vec(front.get_normal(), Vec3::new(0., 0., 1.));

        let back_ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(0., 0., 2.));
        let back = HitRecord::new(
            &back_ray,
            0.5,
            Vec3::new(0., 0., 1.),
            0.25,
            0.75,
            INVISIBLE_PTR,
        );
        assert!(!back.is_front_face());
        assert_vec(back.get_normal(), Vec3::new(0., 0., -1.));
    }

    #[test]
    fn bounded_hit_uses_aabb_as_a_gate_not_as_the_surface_hit() {
        let sphere = Sphere::new(Point3::zero(), 1., INVISIBLE_PTR);
        let box_only = Ray::new(Point3::new(-3., 0.9, 0.9), Vec3::new(1., 0., 0.));
        assert!(sphere.is_aabbox_hit(&box_only, 0. ..=f64::INFINITY));
        assert!(sphere.bounded_hit(&box_only, 0. ..=f64::INFINITY).is_none());

        let surface_hit = Ray::new(Point3::new(-3., 0., 0.), Vec3::new(1., 0., 0.));
        let record = sphere
            .bounded_hit(&surface_hit, 0. ..=f64::INFINITY)
            .unwrap();
        assert_close(record.get_t(), 2.);
        assert_point(record.get_p(), Point3::new(-1., 0., 0.));
    }
}
