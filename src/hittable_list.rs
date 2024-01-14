use std::{any::TypeId, collections::HashMap, ops::RangeInclusive};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    hittable_list::raw::RawHittableVec,
    ray::Ray,
};

mod raw {
    use std::{marker::PhantomData, ops::RangeInclusive};

    use crossbeam::atomic::AtomicCell;

    use crate::{
        entities::{AABBox, Bounded},
        hittable::{BoundedHittable, HitRecord, Hittable},
        ray::Ray,
    };

    pub struct RawHittableVec {
        ptr: *mut u8,
        len: usize,
        cap: usize,
        cached_aabox: AtomicCell<Option<AABBox>>,
        hit: for<'a> fn(
            *mut u8,
            usize,
            PhantomData<&'a ()>,
            &Ray,
            RangeInclusive<f64>,
        ) -> Option<HitRecord<'a>>,
        get_aabbox: fn(*mut u8, usize) -> AABBox,
    }
    // u8 is Send and Sync
    unsafe impl Send for RawHittableVec {}
    unsafe impl Sync for RawHittableVec {}

    fn hit<'a, T>(
        ptr: *mut u8,
        len: usize,
        _phantom: PhantomData<&'a ()>,
        ray: &Ray,
        range: RangeInclusive<f64>,
    ) -> Option<HitRecord<'a>>
    where
        T: BoundedHittable,
    {
        let &start = range.start();
        let &end = range.end();
        unsafe { std::slice::from_raw_parts(ptr.cast_const().cast::<T>(), len) }
            .iter()
            .filter_map(|obj| {
                (obj.is_aabbox_hit(ray, start..=end))
                    .then(|| obj.hit(ray, start..=end))
                    .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }

    fn get_aabbox<T>(ptr: *mut u8, len: usize) -> AABBox
    where
        T: Bounded,
    {
        unsafe { std::slice::from_raw_parts(ptr.cast_const().cast::<T>(), len) }
            .iter()
            .map(|obj| obj.get_aabbox())
            .reduce(|mut acc, e| {
                acc.enclose(e);
                acc
            })
            .expect("Vec shouldn't be empty")
    }

    impl RawHittableVec {
        pub fn new<T>() -> Self
        where
            T: BoundedHittable,
        {
            let mut vec = Vec::<T>::new();
            let ptr = vec.as_mut_ptr();
            let len = vec.len();
            let cap = vec.capacity();
            std::mem::forget(vec);
            Self {
                ptr: ptr.cast(),
                len,
                cap,
                hit: hit::<T>,
                get_aabbox: get_aabbox::<T>,
                cached_aabox: AtomicCell::new(None),
            }
        }

        pub unsafe fn add<T: BoundedHittable>(&mut self, object: T) {
            let mut vec = unsafe { Vec::from_raw_parts(self.ptr.cast(), self.len, self.cap) };
            vec.push(object);
            self.ptr = vec.as_mut_ptr().cast();
            self.len = vec.len();
            self.cap = vec.capacity();
            std::mem::forget(vec);
            self.cached_aabox.store(None);
        }
    }
    impl Hittable for RawHittableVec {
        fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
            (self.hit)(self.ptr, self.len, PhantomData, r, range)
        }
    }

    impl Bounded for RawHittableVec {
        fn get_aabbox(&self) -> AABBox {
            if let Some(aabox) = self.cached_aabox.load() {
                return aabox;
            }
            let aabox = (self.get_aabbox)(self.ptr, self.len);
            self.cached_aabox.store(Some(aabox));
            aabox
        }
    }

    impl BoundedHittable for RawHittableVec {}
}
#[derive(Default)]
pub struct HittableList(HashMap<TypeId, RawHittableVec>);

impl HittableList {
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add<T>(&mut self, object: T)
    where
        T: BoundedHittable,
    {
        let key = object.type_id();
        // SAFETY: as the key is the TypeId of type T, it's safe to use add
        unsafe {
            self.0
                .entry(key)
                .or_insert(RawHittableVec::new::<T>())
                .add(object);
        }
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        let &start = range.start();
        let &end = range.end();
        self.0
            .iter()
            .filter_map(|(_, obj)| {
                obj.hit(r, start..=end)
                // (obj.is_aabbox_hit(r, start..=end))
                //     .then(|| obj.hit(r, start..=end))
                //     .flatten()
            })
            .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
    }
}
