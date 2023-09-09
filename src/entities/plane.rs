use std::ops::RangeInclusive;

use crate::{
    hittable::{HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

pub struct Plane {
    point: Point3,
    normal: Vec3,
    mat_ptr: Box<dyn Material>,
}

impl Plane {
    pub fn new(point: Point3, normal: Vec3, mat_ptr: Box<dyn Material>) -> Self {
        Self {
            point,
            normal: normal.unit_vec(),
            mat_ptr,
        }
    }
}

impl Hittable for Plane {
    #[inline(never)]
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord> {
        let denom = r.get_direction().dot(self.normal);
        (denom > 0.001).then_some(())?;
        let t = (r.get_origin() - self.point).dot(self.normal) / denom;
        (range.contains(&t)).then(|| HitRecord::new(r, t, self.normal, self.mat_ptr.as_ref()))
    }
}
