use std::ops::RangeInclusive;

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3}, entities::Bounded,
};

use super::AABBox;

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
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord> {
        let denom = r.get_direction().dot(self.normal);
        (denom > 0.001).then_some(())?;
        let t = (r.get_origin() - self.point).dot(self.normal) / denom;
        (range.contains(&t)).then(|| HitRecord::new(r, t, self.normal, self.mat_ptr.as_ref()))
    }
}

impl Bounded for Plane {
    fn get_aabbox(&self) -> AABBox {
        let (x_min, x_max) = if self.normal.get_z().abs() < f64::EPSILON
            && self.normal.get_y().abs() < f64::EPSILON
        {
            (0., 0.)
        } else {
            (-f64::INFINITY, f64::INFINITY)
        };
        let (y_min, y_max) = if self.normal.get_x().abs() < f64::EPSILON
            && self.normal.get_z().abs() < f64::EPSILON
        {
            (0., 0.)
        } else {
            (-f64::INFINITY, f64::INFINITY)
        };
        let (z_min, z_max) = if self.normal.get_x().abs() < f64::EPSILON
            && self.normal.get_y().abs() < f64::EPSILON
        {
            (0., 0.)
        } else {
            (-f64::INFINITY, f64::INFINITY)
        };
        AABBox::new(x_min, x_max, y_min, y_max, z_min, z_max)
    }
}

impl BoundedHittable for Plane {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.hit(r, range).is_some()
    }
}
