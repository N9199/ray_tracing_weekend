use std::ops::RangeInclusive;

use crate::{
    hittable::{HitRecord, Hittable},
    ray::Ray,
};

#[derive(Default)]
pub struct HittableList(Vec<Box<dyn Hittable>>);

impl HittableList {
    pub fn new(object: Box<dyn Hittable>) -> Self {
        Self(vec![object])
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.0.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let mut output: Option<HitRecord> = None;
        for obj in self.0.iter() {
            let range = (*range.start())..=output.as_ref().map_or(*range.end(), |v| v.get_t());
            if let Some(curr) = obj.hit(r, range) {
                output = Some(curr);
            }
        }
        output
    }
}
