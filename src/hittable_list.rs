use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    ops::RangeInclusive,
};

use crate::{
    entities::{get_axis, AABBox, AAPlane, Axis, Bounded, Plane},
    hittable::{BoundedHittable, HitRecord, Hittable},
    hittable_list::raw::RawHittableVec,
    ray::Ray,
};

mod raw;
#[derive(Default, Debug)]
pub struct HittableList {
    values: HashMap<TypeId, RawHittableVec>,
    len: usize,
    aabox: Option<AABBox>,
}

impl HittableList {
    pub fn clear(&mut self) {
        self.values.clear();
        self.len = 0;
        self.aabox = None;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn add<T>(&mut self, object: T)
    where
        T: BoundedHittable + Debug + Any,
    {
        self.aabox
            .get_or_insert_with(|| object.get_aabbox())
            .enclose(&object);
        let key = object.type_id();
        // SAFETY: as the key is the TypeId of type T, it's safe to use add
        unsafe {
            self.values
                .entry(key)
                .or_insert(RawHittableVec::new::<T>())
                .add(object);
        }
        self.len += 1;
    }

    pub fn split_by(self, plane: AAPlane) -> (Self, Self) {
        let (mut left, mut right) = (Self::default(), Self::default());
        self.values.into_iter().for_each(|(id, obj)| {
            let (obj_left, obj_right) = obj.split_by(plane);
            if obj_right.len() > 0 {
                let aabox = obj_right.get_aabbox();
                right.len += obj_right.len();
                right.values.insert(id, obj_right);
                right.aabox.get_or_insert(aabox.clone()).enclose(&aabox);
            }
            if obj_left.len() > 0 {
                let aabox = obj_left.get_aabbox();
                left.len += obj_left.len();
                left.values.insert(id, obj_left);
                left.aabox.get_or_insert(aabox.clone()).enclose(&aabox);
            }
        });
        (right, left)
    }

    pub fn split_at_half(self) -> (Self, Self, AAPlane) {
        if self.len() == 0 {
            return (
                Self::default(),
                Self::default(),
                AAPlane {
                    coord: 0.,
                    axis: Axis::X,
                },
            );
        }
        // First find best axis
        let mut best_separator = (usize::MAX, f64::INFINITY, Axis::X, 0.);
        for axis in get_axis() {
            let mut temp_vec = self
                .iter_aaboxes()
                .map(|aabox| aabox.axis(axis))
                .collect::<Vec<_>>();
            temp_vec.sort_by(|r1, r2| {
                r1.start()
                    .total_cmp(r2.start())
                    .then(r1.end().total_cmp(r2.end()))
            });
            // partition_point < temp_vec.len()/2
            // as we search for the first element which is strictly less than the element at temp_vec.len()/2
            let partition_point = temp_vec.partition_point(|v| {
                v.start()
                    .total_cmp(temp_vec[temp_vec.len() / 2].start())
                    .is_lt()
            });
            let bbox_axis_size = temp_vec.last().unwrap().end() - temp_vec.first().unwrap().start();
            // Should be at least 1
            dbg!(
                best_separator,
                bbox_axis_size,
                partition_point,
                axis,
                temp_vec.len(),
                &temp_vec
            );
            if (best_separator.0, -best_separator.1)
                > (temp_vec.len() - 2 * partition_point, -bbox_axis_size)
            {
                best_separator = (
                    temp_vec.len() - 2 * partition_point,
                    bbox_axis_size,
                    axis,
                    *temp_vec[temp_vec.len() / 2].start(),
                )
            }
        }
        let plane = AAPlane {
            coord: best_separator.3,
            axis: best_separator.2,
        };
        let len = self.len();
        let (left, right) = self.split_by(plane);
        debug_assert_ne!(left.len(), len);
        debug_assert_ne!(right.len(), len);
        (left, right, plane)
    }

    pub fn iter_aaboxes<'a>(&'a self) -> impl Iterator<Item = AABBox> + 'a {
        self.values.values().flat_map(|v| v.iter_aaboxes())
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        self.values
            .iter()
            .filter_map(|(_, obj)| {
                // obj.hit(r, start..=end)
                (obj.is_aabbox_hit(r, start..=end))
                    .then(|| obj.hit(r, start..=end))
                    .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }
}

impl Bounded for HittableList {
    fn get_aabbox(&self) -> AABBox {
        self.aabox.unwrap_or(AABBox::new(0., 0., 0., 0., 0., 0.))
    }

    fn get_surface_area(&self) -> f64 {
        self.values
            .values()
            .map(RawHittableVec::get_surface_area)
            .sum()
    }
}

impl BoundedHittable for HittableList {}
