use std::ops::RangeInclusive;

use crate::{
    entities::Bounded,
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::Point3,
};

use super::AABBox;

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat_ptr: Box<dyn Material>,
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord> {
        let oc = r.get_origin() - self.center;
        let a = r.get_direction().length_squared();
        let half_b = r.get_direction().dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        (discriminant > 0.).then_some(())?;

        let sqrt_discriminant = discriminant.sqrt();
        let t = {
            let root = (-half_b - sqrt_discriminant) / a;
            if !(*range.start() <= root && root <= *range.end()) {
                let root = (-half_b + sqrt_discriminant) / a;
                (*range.start() <= root && root <= *range.end()).then_some(root)?
            } else {
                root
            }
        };

        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        Some(HitRecord::new(r, t, outward_normal, self.mat_ptr.as_ref()))
    }
}

impl Bounded for Sphere {
    fn get_aabbox(&self) -> AABBox {
        AABBox::new(
            self.center.get_x() - self.radius,
            self.center.get_x() + self.radius,
            self.center.get_y() - self.radius,
            self.center.get_y() + self.radius,
            self.center.get_z() - self.radius,
            self.center.get_z() + self.radius,
        )
    }
}

impl BoundedHittable for Sphere {}
