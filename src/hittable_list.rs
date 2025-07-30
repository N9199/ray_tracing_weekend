use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    ops::RangeInclusive,
};

use rand::seq::IteratorRandom;

use crate::{
    entities::{get_axis, AABBox, AAPlane, Axis, Bounded},
    geometry::vec3::{Point3, Vec3},
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

    pub const fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add<T>(&mut self, object: T)
    where
        T: BoundedHittable + Debug + Any,
    {
        self.aabox = self
            .aabox
            .map_or_else(|| object.get_aabbox(), |b| b.enclose(&object))
            .into();
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
                right.aabox = right.aabox.map_or(aabox, |b| b.enclose(&aabox)).into();
            }
            if obj_left.len() > 0 {
                let aabox = obj_left.get_aabbox();
                left.len += obj_left.len();
                left.values.insert(id, obj_left);
                left.aabox = left.aabox.map_or(aabox, |b| b.enclose(&aabox)).into();
            }
        });
        (right, left)
    }

    pub fn best_split(self) -> (Self, Self, AAPlane) {
        if self.is_empty() {
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
                .iter_bounded()
                .map(|bounded| bounded.get_aabbox().axis(axis))
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
            // dbg!(
            //     best_separator,
            //     bbox_axis_size,
            //     partition_point,
            //     axis,
            //     temp_vec.len(),
            //     &temp_vec
            // );
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

    pub fn iter_bounded(&self) -> impl Iterator<Item = &'_ dyn Bounded> + '_ {
        self.values.values().flat_map(|v| v.iter_bounded())
    }

    pub fn iter_hittable(&self) -> impl Iterator<Item = &'_ dyn Hittable> + '_ {
        self.values.values().flat_map(|v| v.iter_hittable())
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("HittableList");
        let &start = range.start();
        let &end = range.end();
        self.values
            .iter()
            .filter_map(|(_, obj)| {
                // obj.hit(r, start..=end)
                (obj.is_aabbox_hit(r, start..=end))
                    .then(|| {
                        // dbg!(r);
                        obj.hit(r, start..=end)
                    })
                    .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        self.iter_hittable().fold(0., move |accum, val| {
            accum + val.pdf_value(origin, direction)
        }) / (self.len() as f64)
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        self.iter_hittable()
            .choose(rng)
            .expect("HittableList shouldn't be empty")
            .random(origin, rng)
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

impl<T> Extend<T> for HittableList
where
    T: BoundedHittable + Debug + Any,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        iter.into_iter().for_each(|v| self.add(v));
    }
}
