use std::{fmt::Debug, ops::RangeInclusive};

use crate::{
    geometry::vec3::{Point3, Vec3},
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

use super::{AABBox, Bounded, Quad, get_axis};

#[derive(Debug, Clone)]
pub struct Cuboid {
    quads: [Quad; 6],
}

impl Cuboid {
    pub fn new<T>(p: Point3, q: Point3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabox = AABBox::from(p).enclose(&q);
        let min_p = Point3::new_array(get_axis().map(|axis| *aabox.axis(axis).start()));
        let max_p = Point3::new_array(get_axis().map(|axis| *aabox.axis(axis).end()));
        let delta = max_p - min_p;
        let dx = Vec3::new(delta.get_x(), 0., 0.);
        let dy = Vec3::new(0., delta.get_y(), 0.);
        let dz = Vec3::new(0., 0., delta.get_z());
        let quads = [
            Quad::new(min_p, dx, dy, mat_ptr.clone()),
            Quad::new(min_p, dy, dz, mat_ptr.clone()),
            Quad::new(min_p, dx, dz, mat_ptr.clone()),
            Quad::new(max_p, -dx, -dy, mat_ptr.clone()),
            Quad::new(max_p, -dy, -dz, mat_ptr.clone()),
            Quad::new(max_p, -dx, -dz, mat_ptr.clone()),
        ];
        Self { quads }
    }
}

impl Hittable for Cuboid {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        self.quads
            .iter()
            .filter_map(|q| q.hit(r, range.clone()))
            .min_by(|hit1, hit2| hit1.get_t().total_cmp(&hit2.get_t()))
    }
}

impl Bounded for Cuboid {
    fn get_aabbox(&self) -> AABBox {
        self.quads
            .iter()
            .fold(None, |accum, v| {
                accum
                    .map_or_else(|| v.get_aabbox(), |b: AABBox| b.enclose(v))
                    .into()
            })
            .unwrap()
    }

    fn get_surface_area(&self) -> f64 {
        self.quads.iter().map(|q| q.get_surface_area()).sum()
    }
}

impl BoundedHittable for Cuboid {}
