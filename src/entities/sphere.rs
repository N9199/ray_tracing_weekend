use std::{
    f64::consts::{PI, TAU},
    ops::{Div, RangeInclusive},
    sync::Arc,
};

use crate::{
    entities::Bounded,
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::Point3,
};

use super::AABBox;

#[derive(Debug)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: Arc<dyn Material>,
    aabox: AABBox,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<dyn Material>) -> Self {
        Sphere {
            center,
            radius,
            mat_ptr,
            aabox: AABBox::new(
                center.get_x() - radius,
                center.get_x() + radius,
                center.get_y() - radius,
                center.get_y() + radius,
                center.get_z() - radius,
                center.get_z() + radius,
            ),
        }
    }

    pub fn get_sphere_uv(point: Point3) -> (f64, f64) {
        (
            (-point.get_z()).atan2(point.get_x()).div(TAU),
            point.get_y().acos().div(PI),
        )
    }
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
        let (u, v) = Sphere::get_sphere_uv(outward_normal);
        Some(HitRecord::new(
            r,
            t,
            outward_normal,
            u,
            v,
            self.mat_ptr.as_ref(),
        ))
    }
}

impl Bounded for Sphere {
    fn get_aabbox(&self) -> AABBox {
        self.aabox
    }

    fn get_surface_area(&self) -> f64 {
        4. * PI * self.radius * self.radius
    }
}

impl BoundedHittable for Sphere {}
