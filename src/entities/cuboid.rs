use std::{ops::RangeInclusive, sync::Arc};

use crate::{
    geometry::vec3::{Point3, Vec3},
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
};

use super::{get_axis, AABBox, Bounded, Quad};

#[derive(Debug, Clone)]
pub struct Cuboid {
    quads: [Quad; 6],
}

impl Cuboid {
    pub fn new(p: Point3, q: Point3, mat_ptr: Arc<dyn Material>) -> Self {
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
