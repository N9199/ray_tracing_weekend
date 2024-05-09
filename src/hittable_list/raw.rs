mod type_shit {
    use std::{alloc::Layout, fmt::Debug};

    use crate::{
        entities::{AABBox, AAPlane, Bounded},
        hittable::{BoundedHittable, Hittable},
        utils::slice::Slice,
    };

    use super::RawHittableVec;

    pub struct Functions {
        pub(crate) into_hittable: unsafe fn(*const Slice<u8>) -> *const dyn Hittable,
        pub(crate) into_bounded: unsafe fn(*const Slice<u8>) -> *const dyn Bounded,
        pub(crate) into_debug: unsafe fn(*const Slice<u8>) -> *const dyn Debug,
        pub(crate) aabox_shim: unsafe fn(*const u8) -> (AABBox, *const u8),
        pub(crate) drop_shim: unsafe fn(*mut u8, usize, usize),
        pub(crate) split_by: fn(*mut u8, usize, usize, AAPlane) -> (RawHittableVec, RawHittableVec),
    }

    pub trait RawHittableVecFns {
        const FUNCTIONS: Functions;
    }

    impl<T> RawHittableVecFns for T
    where
        T: Debug + BoundedHittable + 'static,
    {
        const FUNCTIONS: Functions = Functions {
            into_hittable: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice)
                    as *const dyn Hittable
            },
            into_bounded: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice)
                    as *const dyn Bounded
            },
            into_debug: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice) as *const dyn Debug
            },
            aabox_shim: |ptr| unsafe {
                (
                    ptr.cast::<T>().as_ref().unwrap().get_aabbox(),
                    ptr.wrapping_add(std::mem::size_of::<T>()),
                )
            },
            drop_shim: |ptr, len, cap| unsafe {
                let temp_ptr = ptr;
                if cap != 0 {
                    if std::mem::needs_drop::<T>() {
                        let mut lambda = {
                            let mut len = len;
                            let ptr = ptr.cast::<T>();
                            move || {
                                if len == 0 {
                                    None
                                } else {
                                    len -= 1;
                                    Some(std::ptr::read(ptr.add(len)))
                                }
                            }
                        };
                        while let Some(_) = lambda() {}
                    }
                    let layout = Layout::array::<T>(cap).unwrap();
                    std::alloc::dealloc(temp_ptr, layout);
                }
            },
            split_by: |ptr, mut len, _, plane| {
                dbg!(plane);
                let (mut left, mut right) =
                    (RawHittableVec::new::<T>(), RawHittableVec::new::<T>());
                while len != 0 {
                    len -= 1;
                    let val = unsafe { std::ptr::read(ptr.cast::<T>().cast_const().add(len)) };
                    if !val.get_aabbox().right_of(plane) {
                        dbg!("left", val.get_aabbox());
                        unsafe {
                            left.add(val);
                        }
                    } else {
                        dbg!("right", val.get_aabbox());
                        unsafe {
                            right.add(val);
                        }
                    }
                }
                (left, right)
            },
        };
    }
}
use std::{fmt::Debug, marker::PhantomData, ops::RangeInclusive};

use crossbeam::atomic::AtomicCell;

use crate::{
    entities::{AABBox, AAPlane, Bounded},
    hittable::{BoundedHittable, HitRecord, Hittable},
    ray::Ray,
};

use self::type_shit::{Functions, RawHittableVecFns};

#[repr(C)]
pub struct RawHittableVec {
    ptr: *mut u8,
    len: usize,
    cap: usize,
    cached_aabox: AtomicCell<Option<AABBox>>,
    fns: &'static Functions,
}
// u8 is Send and Sync
unsafe impl Send for RawHittableVec {}
unsafe impl Sync for RawHittableVec {}

// fn sort_by_axis<T>(ptr: *mut u8, len: usize, axis: Axis)
// where
//     T: BoundedHittable,
// {
//     unsafe { std::slice::from_raw_parts_mut(ptr.cast::<T>(), len) }
//         .sort_by(|a, b| a.get_aabbox().compare_by_axis(&b.get_aabbox(), axis));
// }

impl RawHittableVec {
    pub fn new<T>() -> Self
    where
        T: RawHittableVecFns + BoundedHittable + Debug + 'static,
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
            cached_aabox: AtomicCell::new(None),
            fns: &T::FUNCTIONS,
        }
    }

    pub(crate) unsafe fn add<T: BoundedHittable>(&mut self, object: T) {
        let mut vec = unsafe { Vec::from_raw_parts(self.ptr.cast(), self.len, self.cap) };
        let bbox = object.get_aabbox();
        vec.push(object);
        self.ptr = vec.as_mut_ptr().cast();
        self.len = vec.len();
        self.cap = vec.capacity();
        std::mem::forget(vec);
        let bbox = self.cached_aabox.load().map_or(bbox, |mut value| {
            value.enclose(&bbox);
            value
        });
        self.cached_aabox.store(Some(bbox));
    }

    pub fn split_by(mut self, plane: AAPlane) -> (Self, Self) {
        let out = (self.fns.split_by)(self.ptr, self.len, self.cap, plane);
        // ! TODO Verify
        self.len = 0;
        out
    }

    pub fn len(&self) -> usize {
        self.len
    }

    // pub fn sort_by_axis(&mut self, axis: Axis) {
    //     (self.sort_by_axis)(self.ptr, self.len, axis)
    // }

    pub fn iter_aaboxes(&self) -> RawHittableVecAABoxesIterator<'_> {
        RawHittableVecAABoxesIterator {
            ptr: self.ptr,
            len: self.len,
            fns: self.fns,
            _phantom: PhantomData,
        }
    }
}

impl Hittable for RawHittableVec {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        unsafe {
            (self.fns.into_hittable)(std::mem::transmute(self as *const _))
                .as_ref()
                .unwrap()
        }
        .hit(r, range)
    }
}

impl Bounded for RawHittableVec {
    fn get_aabbox(&self) -> AABBox {
        if let Some(aabox) = self.cached_aabox.load() {
            return aabox;
        }
        let aabox =
            unsafe { (self.fns.into_bounded)(std::mem::transmute(self as *const _)).as_ref() }
                .unwrap()
                .get_aabbox();
        self.cached_aabox.store(Some(aabox));
        aabox
    }

    fn get_surface_area(&self) -> f64 {
        unsafe { ((self.fns.into_bounded)(std::mem::transmute(self as *const _))).as_ref() }
            .unwrap()
            .get_surface_area()
    }
}

impl std::fmt::Debug for RawHittableVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { (self.fns.into_debug)(std::mem::transmute(self as *const _)).as_ref() }
            .unwrap()
            .fmt(f)
    }
}

impl BoundedHittable for RawHittableVec {}

impl Drop for RawHittableVec {
    fn drop(&mut self) {
        unsafe { (self.fns.drop_shim)(self.ptr, self.len, self.cap) };
    }
}

pub struct RawHittableVecAABoxesIterator<'a> {
    ptr: *const u8,
    len: usize,
    fns: &'static Functions,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for RawHittableVecAABoxesIterator<'a> {
    type Item = AABBox;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let (out, ptr) = unsafe { (self.fns.aabox_shim)(self.ptr) };
            self.ptr = ptr;
            self.len -= 1;
            Some(out)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a> ExactSizeIterator for RawHittableVecAABoxesIterator<'a> {}
