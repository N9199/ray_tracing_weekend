#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{
    fmt::Debug,
    ops::{Div, RangeInclusive, Sub},
};

use rand::{Rng as _, distributions::Open01};

use geometry::{
    aabox::AABBox,
    bounded::Bounded,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

#[derive(Debug, Clone)]
pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    mat_ptr: DynMaterial,
    aabbox: AABBox,
    area: f64,
}

impl Quad {
    pub fn new<T>(q: Point3, u: Vec3, v: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabbox = AABBox::from(q + (u + v) * 0.5)
            .enclose(&(q))
            .enclose(&(q + v))
            .enclose(&(q + u))
            .enclose(&(q + u + v));
        let normal = u.cross(v);
        let w = u.cross(v).div(u.cross(v).length_squared());
        let area = normal.length();
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

    fn get_quad_uv(&self, point: Point3) -> (f64, f64) {
        (
            point.sub(self.q).cross(self.v).dot(self.w),
            self.u.cross(point.sub(self.q)).dot(self.w),
        )
    }
}

impl Bounded for Quad {
    fn get_aabbox(&self) -> AABBox {
        self.aabbox
    }

    fn get_surface_area(&self) -> f64 {
        self.u.cross(self.v).length()
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static QUAD_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Quad {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("Quad");
        let denom = r.get_direction().dot(self.normal);
        (denom.abs() > f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.q).dot(self.normal).div(denom);
        (range.contains(&t)).then_some(())?;
        let point = r.at(t);
        let (u, v) = self.get_quad_uv(point);
        const UNIT: RangeInclusive<f64> = (0.)..=1.;
        // dbg!(UNIT.contains(&u), UNIT.contains(&v), u, v, point);
        (UNIT.contains(&u) && UNIT.contains(&v)).then(|| {
            // dbg!("Quad Hit!");

            #[cfg(feature = "hit_counters")]
            QUAD_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            // dbg!(self.mat_ptr.as_ref());
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self.hit(&Ray::new(origin, direction), (0.)..=f64::INFINITY) {
            Some(record) => {
                let distance_squared = record.get_t() * record.get_t() * direction.length_squared();
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
        let p =
            self.q + self.u * rng.sample::<f64, _>(Open01) + self.v * rng.sample::<f64, _>(Open01);
        p - origin
    }
}

impl BoundedHittable for Quad {
    // fn is_aabbox_hit(&self, _: &Ray, _: RangeInclusive<f64>) -> bool {

    //     true
    // }
}
