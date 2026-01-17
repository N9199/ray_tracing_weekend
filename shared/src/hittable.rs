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
    use geometry::aabox::Box3DExt as _;
    use geometry::{aabox::AABBox, aaplane, bounded::Bounded};

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
    pub const fn get_u(&self) -> f64 {
        self.u
    }

    #[inline]
    pub const fn get_v(&self) -> f64 {
        self.v
    }

    #[inline]
    pub const fn get_p(&self) -> Point3 {
        self.p
    }

    #[inline]
    pub const fn get_normal(&self) -> Vec3 {
        self.normal
    }

    #[inline]
    pub const fn get_t(&self) -> f64 {
        self.t
    }

    #[inline]
    pub const fn is_front_face(&self) -> bool {
        self.front_face
    }

    #[inline]
    pub const fn get_material(&self) -> &dyn Material {
        self.mat_ptr
    }

    #[inline]
    pub(crate) const fn get_mut_p(&mut self) -> &mut Point3 {
        &mut self.p
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
