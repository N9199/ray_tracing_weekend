use core::hint::unreachable_unchecked;
use std::{any::TypeId, collections::HashMap, ops::RangeInclusive};

use crate::{
    hittable::{HitRecord, Hittable, HittableArray},
    ray::Ray,
};

#[derive(Default)]
pub struct HittableList(HashMap<TypeId, Box<dyn HittableArray>>);

impl HittableList {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add<T>(&mut self, object: T)
    where
        T: Hittable,
    {
        let key = object.type_id();
        let Some(list) = self
            .0
            .entry(key)
            .or_insert(Box::<Vec<T>>::default() as Box<dyn HittableArray>)
            .as_mut()
            .as_any_mut()
            .downcast_mut::<Vec<T>>()
        else {
            // SAFETY: This is guaranteed to be of type Vec<T>, as it's indexed using it's TypeId
            unsafe { unreachable_unchecked() }
            // unreachable!()
        };
        list.push(object);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        // if self.0.len() > 1000 {
        //     self.0
        //         .par_iter()
        //         .filter_map(|obj| obj.hit(r, start..=end))
        //         .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
        // } else {
        self.0
            .iter()
            .filter_map(|(_, v)| v.hit(r, start..=end))
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
        // }
    }
}
