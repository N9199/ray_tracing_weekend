mod type_shit {
    use std::{alloc::Layout, fmt::Debug};

    use crate::{
        entities::{AABBox, AAPlane, Bounded},
        hittable::{BoundedHittable, Hittable},
        utils::slice::Slice,
    };

    use super::RawHittableVec;

    pub struct Functions {
        pub(crate) slice_into_hittable: unsafe fn(*const Slice<u8>) -> *const dyn Hittable,
        pub(crate) slice_into_bounded: unsafe fn(*const Slice<u8>) -> *const dyn Bounded,
        pub(crate) slice_into_debug: unsafe fn(*const Slice<u8>) -> *const dyn Debug,
        pub(crate) into_hittable: unsafe fn(*const u8) -> *const dyn Hittable,
        pub(crate) into_bounded: unsafe fn(*const u8) -> *const dyn Bounded,
        pub(crate) into_debug: unsafe fn(*const u8) -> *const dyn Debug,
        pub(crate) advance_by_one_shim: unsafe fn(*const u8) -> *const u8,
        pub(crate) drop_shim: unsafe fn(*mut u8, usize, usize),
        pub(crate) split_by: fn(*mut u8, usize, usize, AAPlane) -> (RawHittableVec, RawHittableVec),
    }

    pub trait RawHittableVecFns {
        const FUNCTIONS: Functions;
    }

    impl<T> RawHittableVecFns for T
    where
        T: Debug + BoundedHittable + Sync + Send + 'static,
    {
        const FUNCTIONS: Functions = Functions {
            slice_into_hittable: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice)
                    as *const dyn Hittable
            },
            slice_into_bounded: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice)
                    as *const dyn Bounded
            },
            slice_into_debug: |slice| unsafe {
                std::mem::transmute::<*const Slice<u8>, *const Slice<T>>(slice) as *const dyn Debug
            },
            into_hittable: |ptr| unsafe {
                std::mem::transmute::<*const u8, *const T>(ptr) as *const dyn Hittable
            },
            into_bounded: |ptr| unsafe {
                std::mem::transmute::<*const u8, *const T>(ptr) as *const dyn Bounded
            },
            into_debug: |ptr| unsafe {
                std::mem::transmute::<*const u8, *const T>(ptr) as *const dyn Debug
            },
            advance_by_one_shim: |ptr| unsafe { ptr.wrapping_add(std::mem::size_of::<T>()) },
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
                        #[allow(clippy::redundant_pattern_matching)]
                        while let Some(_) = lambda() {}
                    }
                    let layout = Layout::array::<T>(cap).unwrap();
                    std::alloc::dealloc(temp_ptr, layout);
                }
            },
            split_by: |ptr, mut len, _, plane| {
                // dbg!(plane);
                let (mut left, mut right) =
                    (RawHittableVec::new::<T>(), RawHittableVec::new::<T>());
                while len != 0 {
                    len -= 1;
                    let val = unsafe { std::ptr::read(ptr.cast::<T>().cast_const().add(len)) };
                    if !val.get_aabbox().right_of(plane) {
                        // dbg!("left", val.get_aabbox());
                        unsafe {
                            left.add(val);
                        }
                    } else {
                        // dbg!("right", val.get_aabbox());
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
    hittable_list::raw::{
        bounded_iterator::RawHittableVecBoundedIterator,
        hittable_iterator::RawHittableVecHittableIterator,
    },
    ray::Ray,
    utils::slice::Slice,
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
// To create a RawHittableVec with `new` you have to guarantee that `T` is Send and Sync,
// as this is "essentially" a `Vec<T>` with `T` erased, it also implements Send and Sync
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
    pub const fn new<T>() -> Self
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
        let bbox = self
            .cached_aabox
            .load()
            .map_or(bbox, |value| value.enclose(&bbox));
        self.cached_aabox.store(Some(bbox));
    }

    pub fn split_by(mut self, plane: AAPlane) -> (Self, Self) {
        let out = (self.fns.split_by)(self.ptr, self.len, self.cap, plane);
        // ! TODO Verify
        self.len = 0;
        out
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    // pub fn sort_by_axis(&mut self, axis: Axis) {
    //     (self.sort_by_axis)(self.ptr, self.len, axis)
    // }

    pub const fn iter_bounded(&self) -> impl Iterator<Item = &'_ dyn Bounded> + '_ {
        RawHittableVecBoundedIterator {
            ptr: self.ptr,
            len: self.len,
            fns: self.fns,
            _phantom: PhantomData,
        }
    }

    pub const fn iter_hittable(&self) -> impl Iterator<Item = &'_ dyn Hittable> + '_ {
        RawHittableVecHittableIterator {
            ptr: self.ptr,
            len: self.len,
            fns: self.fns,
            _phantom: PhantomData,
        }
    }
}

impl Hittable for RawHittableVec {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("RawHittableVec");
        unsafe {
            (self.fns.slice_into_hittable)(std::mem::transmute::<
                *const RawHittableVec,
                *const Slice<u8>,
            >(self as *const _))
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
        let aabox = unsafe {
            (self.fns.slice_into_bounded)(std::mem::transmute::<
                *const RawHittableVec,
                *const Slice<u8>,
            >(self as *const _))
            .as_ref()
        }
        .unwrap()
        .get_aabbox();
        self.cached_aabox.store(Some(aabox));
        aabox
    }

    fn get_surface_area(&self) -> f64 {
        unsafe {
            ((self.fns.slice_into_bounded)(std::mem::transmute::<
                *const RawHittableVec,
                *const Slice<u8>,
            >(self as *const _)))
            .as_ref()
        }
        .unwrap()
        .get_surface_area()
    }
}

impl std::fmt::Debug for RawHittableVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            (self.fns.slice_into_debug)(std::mem::transmute::<
                *const RawHittableVec,
                *const Slice<u8>,
            >(self as *const _))
            .as_ref()
        }
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

mod bounded_iterator {
    use crate::entities::Bounded;

    use std::marker::PhantomData;

    use super::type_shit::Functions;

    pub struct RawHittableVecBoundedIterator<'a> {
        pub(crate) ptr: *const u8,
        pub(crate) len: usize,
        pub(crate) fns: &'static Functions,
        pub(crate) _phantom: PhantomData<&'a ()>,
    }

    impl<'a> Iterator for RawHittableVecBoundedIterator<'a> {
        type Item = &'a dyn Bounded;

        fn next(&mut self) -> Option<Self::Item> {
            if self.len == 0 {
                None
            } else {
                let out_ptr = unsafe { ((self.fns.into_bounded)(self.ptr)) };
                let ptr = unsafe { (self.fns.advance_by_one_shim)(self.ptr) };
                self.ptr = ptr;
                self.len -= 1;
                unsafe { out_ptr.as_ref() }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl ExactSizeIterator for RawHittableVecBoundedIterator<'_> {}
}

mod hittable_iterator {
    use crate::hittable::Hittable;

    use std::marker::PhantomData;

    use super::type_shit::Functions;

    pub struct RawHittableVecHittableIterator<'a> {
        pub(crate) ptr: *const u8,
        pub(crate) len: usize,
        pub(crate) fns: &'static Functions,
        pub(crate) _phantom: PhantomData<&'a ()>,
    }

    impl<'a> Iterator for RawHittableVecHittableIterator<'a> {
        type Item = &'a dyn Hittable;

        fn next(&mut self) -> Option<Self::Item> {
            if self.len == 0 {
                None
            } else {
                let out_ptr = unsafe { ((self.fns.into_hittable)(self.ptr)) };
                let ptr = unsafe { (self.fns.advance_by_one_shim)(self.ptr) };
                self.ptr = ptr;
                self.len -= 1;
                unsafe { out_ptr.as_ref() }
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl ExactSizeIterator for RawHittableVecHittableIterator<'_> {}
}
