use std::{f64::consts::PI, fmt::Debug};

use rand::{distributions::Standard, Rng};

use geometry::{
    onb::Onb,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::Hittable,
    utils::random_utils::{CosineWeightedHemisphere, UnitSphere},
};

pub trait Pdf: Send + Sync + Debug {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3;
}

#[derive(Debug)]
pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _direction: &Vec3) -> f64 {
        1. / (4. * PI)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        rng.sample(UnitSphere)
    }
}

#[derive(Debug)]
pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    #[must_use]
    pub fn new(w: Vec3) -> Self {
        Self { uvw: Onb::new(w) }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.normalize().dot(self.uvw.get_w()) / PI;
        cosine_theta.max(0.)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        self.uvw.transform(rng.sample(CosineWeightedHemisphere))
    }
}

#[derive(Debug)]
pub struct HittablePdf<'a> {
    objects: &'a dyn Hittable,
    origin: Point3,
}

impl<'a> HittablePdf<'a> {
    pub const fn new(objects: &'a dyn Hittable, origin: Point3) -> Self {
        Self { objects, origin }
    }
}

impl Pdf for HittablePdf<'_> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(self.origin, *direction)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        self.objects.random(self.origin, rng)
    }
}

#[derive(Debug)]
pub struct MixturePdf<'a, 'b> {
    pdf1: &'a dyn Pdf,
    pdf2: &'b dyn Pdf,
}

impl<'a, 'b> MixturePdf<'a, 'b> {
    pub const fn new(pdf1: &'a dyn Pdf, pdf2: &'b dyn Pdf) -> Self {
        Self { pdf1, pdf2 }
    }
}

impl Pdf for MixturePdf<'_, '_> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.pdf1.value(direction) * 0.5 + self.pdf2.value(direction) * 0.5
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        if rng.sample::<f64, _>(Standard) < 0.5 {
            self.pdf1.generate(rng)
        } else {
            self.pdf2.generate(rng)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use geometry::{
        test_utils::{assert_close as assert_close_with_tolerance, assert_point, assert_vec},
        vec3::{Point3, Vec3},
    };
    use rand::{rngs::SmallRng, RngCore as _, SeedableRng as _};

    use crate::{
        hittable::{HitRecord, Hittable},
        pdf::{CosinePdf, HittablePdf, MixturePdf, Pdf, SpherePdf},
        ray::Ray,
    };

    const TOLERANCE: f64 = 1e-10;
    const SPHERE_PDF_SEED: u64 = 0x5A17;
    const COSINE_PDF_SEED: u64 = 0x00C0_519E;
    const HITTABLE_PDF_SEED: u64 = 0xD311_6A7E;
    const MIXTURE_PDF_SEED: u64 = 0xA11CE;

    fn same_vector_bits(left: Vec3, right: Vec3) -> bool {
        [left.x.to_bits(), left.y.to_bits(), left.z.to_bits()]
            == [right.x.to_bits(), right.y.to_bits(), right.z.to_bits()]
    }

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    #[derive(Debug)]
    struct ConstantPdf {
        density: f64,
        generated: Vec3,
    }

    impl Pdf for ConstantPdf {
        fn value(&self, _direction: &Vec3) -> f64 {
            self.density
        }

        fn generate(&self, _rng: &mut dyn rand::RngCore) -> Vec3 {
            self.generated
        }
    }

    #[derive(Debug, Default)]
    struct RecordingHittable {
        pdf_args: Mutex<Option<(Point3, Vec3)>>,
        random_origin: Mutex<Option<Point3>>,
        random_draw: Mutex<Option<u64>>,
    }

    impl Hittable for RecordingHittable {
        fn hit(&self, _ray: &Ray, _range: std::ops::RangeInclusive<f64>) -> Option<HitRecord<'_>> {
            None
        }

        fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
            *self.pdf_args.lock().unwrap() = Some((origin, direction));
            0.375
        }

        fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
            *self.random_origin.lock().unwrap() = Some(origin);
            *self.random_draw.lock().unwrap() = Some(rng.next_u64());
            Vec3::new(4., -2., 7.)
        }
    }

    #[test]
    fn sphere_pdf_value_is_constant_and_current_generator_stays_inside_the_unit_ball() {
        let pdf = SpherePdf;
        for direction in [
            Vec3::new(1., 0., 0.),
            Vec3::new(2., -3., 4.),
            Vec3::new(0., 0., 0.),
        ] {
            assert_close(pdf.value(&direction), 1. / (4. * std::f64::consts::PI));
        }

        let mut first_rng = SmallRng::seed_from_u64(SPHERE_PDF_SEED);
        let mut second_rng = SmallRng::seed_from_u64(SPHERE_PDF_SEED);
        for _ in 0..64 {
            let first = pdf.generate(&mut first_rng);
            let second = pdf.generate(&mut second_rng);
            assert_vec(first, second, TOLERANCE);
            assert!(first.x.is_finite() && first.y.is_finite() && first.z.is_finite());
            assert!(first.square_length() < 1., "generated vector: {first:?}");
        }
    }

    #[test]
    fn cosine_pdf_values_are_scale_invariant_and_follow_axis_and_oriented_normals() {
        let axis_pdf = CosinePdf::new(Vec3::new(0., 0., 1.));
        assert_close(
            axis_pdf.value(&Vec3::new(0., 0., 1.)),
            1. / std::f64::consts::PI,
        );
        assert_close(
            axis_pdf.value(&Vec3::new(0., 0., 5.)),
            1. / std::f64::consts::PI,
        );
        assert_close(axis_pdf.value(&Vec3::new(2., 0., 0.)), 0.);
        assert_close(axis_pdf.value(&Vec3::new(0., 0., -1.)), 0.);

        let normal = Vec3::new(1., 2., 2.).normalize();
        let oriented_pdf = CosinePdf::new(normal);
        assert_close(
            oriented_pdf.value(&(normal * 3.)),
            1. / std::f64::consts::PI,
        );
        assert_close(oriented_pdf.value(&(-normal)), 0.);
        let tangent = Vec3::new(2., -1., 0.);
        assert_close(oriented_pdf.value(&tangent), 0.);

        let mut first_rng = SmallRng::seed_from_u64(COSINE_PDF_SEED);
        let mut second_rng = SmallRng::seed_from_u64(COSINE_PDF_SEED);
        for _ in 0..64 {
            let sample = oriented_pdf.generate(&mut first_rng);
            assert_vec(sample, oriented_pdf.generate(&mut second_rng), TOLERANCE);
            assert_close(sample.square_length(), 1.);
            assert!(sample.dot(normal) >= -TOLERANCE);
        }
    }

    #[test]
    fn hittable_pdf_delegates_value_origin_direction_and_rng() {
        let hittable = RecordingHittable::default();
        let origin = Point3::new(3., -4., 2.);
        let direction = Vec3::new(-1., 5., 0.25);
        let pdf = HittablePdf::new(&hittable, origin);
        assert_close(pdf.value(&direction), 0.375);
        let (actual_origin, actual_direction) = hittable.pdf_args.lock().unwrap().unwrap();
        assert_point(actual_origin, origin, TOLERANCE);
        assert_vec(actual_direction, direction, TOLERANCE);

        let mut rng = SmallRng::seed_from_u64(HITTABLE_PDF_SEED);
        let mut expected_rng = SmallRng::seed_from_u64(HITTABLE_PDF_SEED);
        assert_vec(pdf.generate(&mut rng), Vec3::new(4., -2., 7.), 0.);
        assert_point(
            hittable.random_origin.lock().unwrap().unwrap(),
            origin,
            TOLERANCE,
        );
        assert_eq!(
            *hittable.random_draw.lock().unwrap(),
            Some(expected_rng.next_u64())
        );
    }

    #[test]
    fn mixture_pdf_averages_values_generates_from_a_source_and_repeats_for_a_seed() {
        let first = ConstantPdf {
            density: 0.2,
            generated: Vec3::new(1., 2., 3.),
        };
        let second = ConstantPdf {
            density: 0.8,
            generated: Vec3::new(-4., 5., 6.),
        };
        let mixture = MixturePdf::new(&first, &second);
        assert_close(mixture.value(&Vec3::new(2., -1., 4.)), 0.5);

        let mut first_rng = SmallRng::seed_from_u64(MIXTURE_PDF_SEED);
        let mut second_rng = SmallRng::seed_from_u64(MIXTURE_PDF_SEED);
        for _ in 0..64 {
            let sample = mixture.generate(&mut first_rng);
            assert_vec(sample, mixture.generate(&mut second_rng), 0.);
            let is_first = same_vector_bits(sample, first.generated);
            let is_second = same_vector_bits(sample, second.generated);
            assert!(is_first || is_second);
        }
    }
}
