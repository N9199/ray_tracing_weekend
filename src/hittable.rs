use core::ops::RangeInclusive;
use std::fmt::Debug;

use crate::{
    entities::Bounded,
    geometry::vec3::{Point3, Vec3},
    material::Material,
    ray::Ray,
};

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
    pub(crate) const fn get_mut_p(&mut self) -> &mut Vec3 {
        &mut self.p
    }
}

pub trait Hittable: Sync + Send + Debug {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>>;

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        0.
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        Vec3::from([1., 0., 0.])
    }
}

pub trait BoundedHittable: Hittable + Bounded + Debug {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.get_aabbox().is_hit(r, range)
    }
}

impl<T> Hittable for &[T]
where
    T: BoundedHittable,
{
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        self.iter()
            .filter_map(|obj| {
                (obj.is_aabbox_hit(r, start..=end))
                    .then(|| {
                        // dbg!(r);
                        obj.hit(r, start..=end)
                    })
                    .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }
}
