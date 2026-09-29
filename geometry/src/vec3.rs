#[cfg(not(feature = "euclid"))]
mod vec;

#[cfg(feature = "euclid")]
use euclid::{default::Translation3D, Point3D, UnknownUnit, Vector3D};
#[cfg(not(feature = "euclid"))]
pub use vec::Vec3;
#[cfg(not(feature = "euclid"))]
pub type Point3 = Vec3;
#[cfg(not(feature = "euclid"))]
pub type Translation3 = Vec3;

#[cfg(feature = "euclid")]
pub type Vec3 = Vector3D<f64, UnknownUnit>;
#[cfg(feature = "euclid")]
pub type Point3 = Point3D<f64, UnknownUnit>;
#[cfg(feature = "euclid")]
pub type Translation3 = Translation3D<f64>;

pub trait Vec3Ext {
    fn is_near_zero(&self) -> bool;

    #[must_use]
    fn refract(self, other: Self, etai_over_etat: f64) -> Self;

    #[must_use]
    fn unit_vec(self) -> Self;

    #[must_use]
    fn to_array(self) -> [f64; 3];
}

#[cfg(feature = "euclid")]
mod inner {
    use euclid::approxeq::ApproxEq as _;

    use crate::vec3::{Vec3, Vec3Ext};

    impl Vec3Ext for Vec3 {
        fn is_near_zero(&self) -> bool {
            self.approx_eq(&Self::zero())
        }

        fn refract(self, other: Self, etai_over_etat: f64) -> Self {
            let cos_theta = self.dot(-other).min(1.);
            let r_out_perp = (self + other * cos_theta) * etai_over_etat;
            let r_out_parallel = other * (-(1. - r_out_perp.square_length()).sqrt());
            r_out_perp + r_out_parallel
        }

        fn unit_vec(self) -> Self {
            self / self.length()
        }

        fn to_array(self) -> [f64; 3] {
            [self.x, self.y, self.z]
        }
    }
}

#[cfg(all(test, feature = "euclid"))]
mod tests {
    use crate::{
        test_utils::{assert_close, assert_vec},
        vec3::{Vec3, Vec3Ext as _},
    };

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn euclid_vector_arithmetic_dot_cross_and_normalization() {
        let a = Vec3::new(1., 2., 3.);
        let b = Vec3::new(-4., 5., 0.5);
        assert_vec(a + b, Vec3::new(-3., 7., 3.5), TOLERANCE);
        assert_vec(a - b, Vec3::new(5., -3., 2.5), TOLERANCE);
        assert_vec(a * 2., Vec3::new(2., 4., 6.), TOLERANCE);
        assert_close(a.dot(b), b.dot(a), TOLERANCE);
        assert_vec(a.cross(b), -b.cross(a), TOLERANCE);
        let vector = Vec3::new(1., 2., 2.);
        assert_close(vector.unit_vec().length(), 1., TOLERANCE);
        assert_vec(vector.unit_vec() * vector.length(), vector, TOLERANCE);
    }

    #[test]
    fn euclid_vector_extensions_cover_conversion_refraction_and_approximate_zero() {
        let vector = Vec3::new(1., -2., 3.);
        assert_eq!(vector.to_array(), [1., -2., 3.]);
        assert!(Vec3::zero().is_near_zero());
        assert!(!Vec3::new(1e-4, 0., 0.).is_near_zero());
        assert_vec(
            Vec3::new(0., -1., 0.).refract(Vec3::new(0., 1., 0.), 1.),
            Vec3::new(0., -1., 0.),
            TOLERANCE,
        );
        let refracted =
            Vec3::new(0.5, -(3.0_f64).sqrt() / 2., 0.).refract(Vec3::new(0., 1., 0.), 0.8);
        assert!(refracted.to_array().into_iter().all(f64::is_finite));
        assert!(refracted.x > 0.);
    }
}
