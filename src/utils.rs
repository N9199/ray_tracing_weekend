pub mod slice {
    use std::fmt::Debug;

    use crate::{
        entities::{AABBox, Bounded},
        hittable::{BoundedHittable, Hittable},
    };

    #[repr(C)]
    pub struct Slice<T> {
        ptr: *mut T,
        len: usize,
    }

    // impl<T> Slice<T> {
    //     pub fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
    //         Self { ptr, len }
    //     }
    // }
    impl<T> Slice<T>
    where
        T: Bounded + 'static,
    {
        pub fn get_aabboxes(&self) -> impl Iterator<Item = AABBox> {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .map(|v| v.get_aabbox())
        }
    }

    impl<T> Debug for Slice<T>
    where
        T: Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }.fmt(f)
        }
    }

    unsafe impl<T> Sync for Slice<T> {}
    unsafe impl<T> Send for Slice<T> {}

    impl<T> Bounded for Slice<T>
    where
        T: Bounded,
    {
        fn get_aabbox(&self) -> AABBox {
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
                .iter()
                .map(|obj| obj.get_aabbox())
                .reduce(|acc, e| acc.enclose(&e))
                .expect("Slice shouldn't be empty")
        }

        fn get_surface_area(&self) -> f64 {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .map(T::get_surface_area)
                .sum()
        }
    }

    impl<T> Hittable for Slice<T>
    where
        T: BoundedHittable,
    {
        fn hit(
            &self,
            r: &crate::ray::Ray,
            range: std::ops::RangeInclusive<f64>,
        ) -> Option<crate::hittable::HitRecord<'_>> {
            // dbg!("Slice");
            // dbg!(self);
            let &start = range.start();
            let &end = range.end();
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .filter_map(|obj| {
                    (obj.is_aabbox_hit(r, start..=end))
                        .then(|| {
                            // dbg!(r);
                            obj.hit(r, start..=end)
                        })
                        .flatten()
                })
                .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
        }
    }
}

pub mod random_utils {
    use std::f64::consts::PI;

    use rand::{
        distributions::{Open01, Standard, Uniform},
        prelude::Distribution,
        seq::SliceRandom,
    };

    use crate::geometry::vec3::Vec3;

    #[inline]
    pub fn random_f64_2<T: rand::Rng + ?Sized>(rng: &mut T) -> f64 {
        let dist = Uniform::new_inclusive(0.5, 1.);
        rng.sample(dist)
    }

    pub struct UnitSphere;

    impl Distribution<Vec3> for UnitSphere {
        #[inline]
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
            // let theta = rng.sample(Uniform::new_inclusive(0., 2. * core::f64::consts::PI));
            // let closed01 = Uniform::<f64>::new_inclusive(0., 1.);
            // let phi = (2. * rng.sample(closed01) - 1.).acos();
            // let r = rng.sample(closed01).cbrt();
            // Vec3::new(
            //     r * theta.cos() * phi.sin(),
            //     r * theta.sin() * phi.sin(),
            //     r * phi.cos(),
            // )
            loop {
                let mut inner = [(); 3].map(|_| 2. * rng.sample::<f64, _>(Standard) - 1.);
                inner.shuffle(rng);
                let out = Vec3::from(inner);
                if out.length_squared() < 1. {
                    return out;
                }
            }
        }
    }

    pub struct UnitDisk;

    impl Distribution<Vec3> for UnitDisk {
        #[inline]
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
            // let theta = rng.sample(Uniform::new_inclusive(0., 2. * core::f64::consts::PI));
            // let closed01 = Uniform::<f64>::new_inclusive(0., 1.);
            // let r = rng.sample(closed01).sqrt();
            // Vec3::new(r * theta.cos(), r * theta.sin(), 0.)
            loop {
                let out = Vec3::new(
                    2. * rng.sample::<f64, _>(Standard) - 1.,
                    0.,
                    2. * rng.sample::<f64, _>(Standard) - 1.,
                );
                if out.length_squared() < 1. {
                    return out;
                }
            }
        }
    }

    pub struct CosineWeightedHemisphere;

    impl Distribution<Vec3> for CosineWeightedHemisphere {
        #[inline]
        fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
            let r1 = rng.sample::<f64, _>(Standard);
            let r2 = rng.sample::<f64, _>(Standard);

            let phi = 2. * PI * r1;
            let x = phi.cos() * r2.sqrt();
            let y = phi.sin() * r2.sqrt();
            let z = (1. - r2).sqrt();

            Vec3::new(x, y, z)
        }
    }
}
