#[cfg(not(feature = "euclid"))]
mod vec;

#[cfg(feature = "euclid")]
use euclid::{Point3D, UnknownUnit, Vector3D, default::Translation3D};
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
    }
}
