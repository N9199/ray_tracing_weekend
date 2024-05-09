use core::ops::RangeInclusive;
use std::fmt::Debug;

use crate::{
    entities::Bounded,
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
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
    pub fn get_u(&self) -> f64 {
        self.u
    }

    #[inline]
    pub fn get_v(&self) -> f64 {
        self.v
    }

    #[inline]
    pub fn get_p(&self) -> Point3 {
        self.p
    }

    #[inline]
    pub fn get_normal(&self) -> Vec3 {
        self.normal
    }

    #[inline]
    pub fn get_t(&self) -> f64 {
        self.t
    }

    #[inline]
    pub fn is_front_face(&self) -> bool {
        self.front_face
    }

    #[inline]
    pub fn get_material(&self) -> &dyn Material {
        self.mat_ptr
    }
}

pub trait Hittable: Sync + Send {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>>;
}

pub trait BoundedHittable: Hittable + Bounded + Debug {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.get_aabbox().is_hit(r, range)
    }
}

impl<'a, T> Hittable for &'a [T]
where
    T: BoundedHittable,
{
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        self.iter()
            .filter_map(|obj| {
                (obj.is_aabbox_hit(r, start..=end))
                    .then(|| obj.hit(r, start..=end))
                    .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }
}
