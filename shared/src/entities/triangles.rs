#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};

use std::fmt::Debug;
use std::ops::{Div, RangeInclusive, Sub};

use geometry::bounded::Bounded;
use geometry::{
    aabox::AABBox,
    vec3::{Point3, Vec3},
};
use rand::Rng as _;
use rand::distributions::Open01;

use crate::hittable::{BoundedHittable, HitRecord, Hittable};
use crate::material::DynMaterial;
use crate::ray::Ray;

#[cfg(feature = "hit_counters")]
pub(crate) static TRIANGLES_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct Triangle {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    mat_ptr: DynMaterial,
    aabbox: AABBox,
    area: f64,
}

impl Triangle {
    pub fn new<T>(q: Point3, u: Vec3, v: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabbox = AABBox::from_points([(q + (u + v) * 0.5), q, q + v, q + u]);
        let normal = u.cross(v);
        let w = normal.div(normal.square_length());
        let area = normal.length() / 2.;
        let normal = normal / area;
        Self {
            q,
            u,
            v,
            w,
            normal,
            mat_ptr,
            aabbox,
            area,
        }
    }

    fn get_triangle_uv(&self, point: Point3) -> (f64, f64) {
        (
            point.sub(self.q).cross(self.v).dot(self.w),
            self.u.cross(point.sub(self.q)).dot(self.w),
        )
    }
}

impl Bounded for Triangle {
    fn get_aabbox(&self) -> AABBox {
        self.aabbox
    }

    fn get_surface_area(&self) -> f64 {
        self.area
    }
}

impl Hittable for Triangle {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let denom = r.get_direction().dot(self.normal);
        (denom.abs() > f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.q).dot(self.normal).div(denom);
        (range.contains(&t)).then_some(())?;
        let point = r.at(t);
        let (u, v) = self.get_triangle_uv(point);
        const UNIT: RangeInclusive<f64> = (0.)..=1.;
        // dbg!(UNIT.contains(&u), UNIT.contains(&v), u, v, point);
        (UNIT.contains(&(u + v))).then(|| {
            #[cfg(feature = "hit_counters")]
            TRIANGLES_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            // dbg!(self.mat_ptr.as_ref());
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self.hit(&Ray::new(origin, direction), (0.)..=f64::INFINITY) {
            Some(record) => {
                let distance_squared = record.get_t() * record.get_t() * direction.square_length();
                let cosine = direction
                    .dot(record.get_normal())
                    .div(direction.length())
                    .abs();
                distance_squared / (cosine * self.area)
            }
            None => 0.,
        }
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        let mut r1 = rng.sample::<f64, _>(Open01);
        let mut r2 = rng.sample::<f64, _>(Open01);
        if r1 + r2 > 1. {
            r1 = 1. - r1;
            r2 = 1. - r2;
        }
        let p = self.q + self.u * r1 + self.v * r2;
        p - origin
    }
}

impl BoundedHittable for Triangle {}
