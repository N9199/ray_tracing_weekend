#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, RangeInclusive, Sub},
};

use crate::{
    entities::Bounded,
    geometry::vec3::{Point3, Vec3},
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

use super::AABBox;

#[derive(Debug)]
pub struct Plane {
    point: Point3,
    normal: Vec3,
    mat_ptr: DynMaterial,
}

impl Plane {
    pub fn new<T>(point: Point3, normal: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        Self {
            point,
            normal: normal.unit_vec(),
            mat_ptr: mat_ptr.try_into().unwrap(),
        }
    }

    #[inline(never)]
    pub fn get_plane_uv(&self, point: Point3) -> (f64, f64) {
        const V: Vec3 = Vec3::new(0., 1., 0.);
        let theta = f64::atan2(self.normal.cross(V).length(), self.normal.dot(V));
        if theta <= f64::EPSILON {
            return (point.get_x(), point.get_z());
        }
        let k = self.normal.cross(V).unit_vec();
        let vec_to_rotate = point - self.point;
        let rotated_vec = vec_to_rotate
            .mul(theta.cos())
            .add(k.cross(vec_to_rotate).mul(theta.sin()))
            .add(k.mul(k.dot(vec_to_rotate)).mul((1.).sub(theta.cos())));
        (rotated_vec.get_x().fract(), rotated_vec.get_z().fract())
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static PLANE_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Plane {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let denom = r.get_direction().dot(self.normal);
        (denom > f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.point).dot(self.normal).div(denom);
        let point = r.at(t);
        let (u, v) = self.get_plane_uv(point);
        if !(u.is_finite() && v.is_finite()) {
            panic!("u = {u}, v = {v}, point = {point:?}");
        }
        (range.contains(&t)).then(|| {
            #[cfg(feature = "hit_counters")]
            PLANE_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
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
