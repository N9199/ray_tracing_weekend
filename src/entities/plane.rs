use std::{
    ops::{Add, Div, Mul, RangeInclusive, Sub},
    sync::Arc,
};

use crate::{
    entities::Bounded,
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
    vec3::{Point3, Vec3},
};

use super::AABBox;

#[derive(Debug)]
pub struct Plane {
    point: Point3,
    normal: Vec3,
    mat_ptr: Arc<dyn Material>,
}

impl Plane {
    pub fn new(point: Point3, normal: Vec3, mat_ptr: Arc<dyn Material>) -> Self {
        Self {
            point,
            normal: normal.unit_vec(),
            mat_ptr,
        }
    }

    pub fn get_plane_uv(&self, point: Point3) -> (f64, f64) {
        const V: Vec3 = Vec3::new(0., 1., 0.);
        let theta = f64::atan2(self.normal.cross(V).length(), self.normal.dot(V));
        let k = self.normal.cross(V).unit_vec();
        let vec_to_rotate = point - self.point;
        let rotated_vec = vec_to_rotate
            .mul(theta.cos())
            .add(k.cross(vec_to_rotate).mul(theta.sin()))
            .add(k.mul(k.dot(vec_to_rotate)).mul((1.).sub(theta.cos())));
        (rotated_vec.get_x(), rotated_vec.get_z())
    }
}
impl Hittable for Plane {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord> {
        let denom = r.get_direction().dot(self.normal);
        let t = -(r.get_origin() - self.point).dot(self.normal).div(denom);
        let point = r.at(t);
        let (u, v) = (point.get_x(), point.get_z());
        (range.contains(&t)).then(|| HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref()))
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

    fn get_surface_area(&self) -> f64 {
        f64::INFINITY
    }
}

impl BoundedHittable for Plane {
    fn is_aabbox_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        self.hit(r, range).is_some()
    }
}
