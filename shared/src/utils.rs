pub mod slice {
    use std::{fmt::Debug, ops::RangeInclusive};

    #[cfg(feature = "euclid")]
    use geometry::aabbox::Box3DExt as _;
    use geometry::{aabbox::AABBox, bounded::Bounded};

    use crate::{
        hittable::{BoundedHittable, HitRecord, Hittable},
        ray::Ray,
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
                .map(Bounded::get_aabbox)
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
                .map(Bounded::get_aabbox)
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
        fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
            let &start = range.start();
            let &end = range.end();
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .filter_map(move |obj| obj.bounded_hit(r, start..=end))
                .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
        }
    }
}

pub mod random_utils {
    use std::{f64::consts::PI, range::RangeInclusive};

    use rand::{
        distributions::{Standard, Uniform},
        prelude::Distribution,
        seq::SliceRandom,
    };

    use geometry::vec3::Vec3;

    pub const UNIT: RangeInclusive<f64> = RangeInclusive {
        start: 0.,
        last: 1.,
    };

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
                let mut inner = [(); 3].map(|()| 2. * rng.sample::<f64, _>(Standard) - 1.);
                inner.shuffle(rng);
                let out = Vec3::from(inner);
                if out.square_length() < 1. {
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
                if out.square_length() < 1. {
                    return out;
                }
            }
        }
    }

    pub struct CosineWeightedHemisphere;

    impl Distribution<Vec3> for CosineWeightedHemisphere {
        // TODO use Malley's method
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

#[cfg(test)]
mod tests {
    use geometry::test_utils::{assert_close, assert_vec};
    use rand::{rngs::SmallRng, Rng as _, SeedableRng as _};

    use crate::utils::random_utils::{CosineWeightedHemisphere, UnitSphere};

    const TOLERANCE: f64 = 1e-10;
    const SEED: u64 = 0x000C_051E;
    const SAMPLE_COUNT: usize = 10_000;

    #[test]
    fn unit_sphere_sampler_is_reproducible_and_returns_points_inside_the_unit_ball() {
        let mut first_rng = SmallRng::seed_from_u64(SEED);
        let mut second_rng = SmallRng::seed_from_u64(SEED);
        for _ in 0..128 {
            let first = first_rng.sample(UnitSphere);
            let second = second_rng.sample(UnitSphere);
            assert_vec(first, second, TOLERANCE);
            assert!(first.x.is_finite() && first.y.is_finite() && first.z.is_finite());
            assert!(first.square_length() < 1., "generated vector: {first:?}");
        }
    }

    #[test]
    fn cosine_hemisphere_sampler_is_reproducible_unit_length_and_cosine_weighted() {
        let mut first_rng = SmallRng::seed_from_u64(SEED);
        let mut second_rng = SmallRng::seed_from_u64(SEED);
        let mut sum_local_z = 0.;
        for _ in 0..SAMPLE_COUNT {
            let first = first_rng.sample(CosineWeightedHemisphere);
            let second = second_rng.sample(CosineWeightedHemisphere);
            assert_vec(first, second, TOLERANCE);
            assert!(first.x.is_finite() && first.y.is_finite() && first.z.is_finite());
            assert_close(first.square_length(), 1., TOLERANCE);
            assert!(first.z >= 0.);
            sum_local_z += first.z;
        }
        let mean_local_z = sum_local_z / SAMPLE_COUNT as f64;
        assert!(
            (mean_local_z - 2. / 3.).abs() < 0.02,
            "seed {SEED:#x}, {SAMPLE_COUNT} samples: mean local-Z {mean_local_z}, expected 2/3"
        );
    }
}
