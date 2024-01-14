use core::{any::Any, ops::RangeInclusive};

use crate::{
    entities::{AABBox, Bounded},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub struct HitRecord<'a> {
    p: Point3,
    normal: Vec3,
    t: f64,
    front_face: bool,
    mat_ptr: &'a dyn Material,
}

impl<'a> HitRecord<'a> {
    #[inline]
    pub fn new(r: &Ray, t: f64, outward_normal: Vec3, mat_ptr: &'a dyn Material) -> Self {
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
        }
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

pub trait Hittable: Sync + Send + Any {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>>;
}

pub trait BoundedHittable: Hittable + Bounded {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.get_aabbox().is_hit(r, range)
    }
}

pub trait HittableArray: Hittable + Sync + Send {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl<T> Hittable for Vec<T>
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
