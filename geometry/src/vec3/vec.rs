use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::{
    iter::Sum,
    ops::{DivAssign, Index, IndexMut},
};

use crate::{aabbox::AABBox, bounded::Bounded, vec3::Point3};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }

    #[inline]
    #[must_use]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    #[must_use]
    pub const fn new_array(inner: [f64; 3]) -> Self {
        Self {
            x: inner[0],
            y: inner[1],
            z: inner[2],
        }
    }

    #[inline]
    #[must_use]
    pub const fn inner(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    #[inline]
    #[must_use]
    pub const fn to_array(self) -> [f64; 3] {
        self.inner()
    }

    #[inline]
    #[must_use]
    pub fn length(self) -> f64 {
        self.square_length().sqrt()
    }

    #[inline]
    #[must_use]
    pub fn square_length(self) -> f64 {
        self.dot(self)
    }

    #[inline]
    #[must_use]
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        Self::new_array([
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        ])
    }

    #[inline]
    #[must_use]
    pub fn unit_vec(self) -> Self {
        self / self.length()
    }

    #[inline]
    #[must_use]
    pub fn normalize(self) -> Self {
        self.unit_vec()
    }

    #[inline]
    #[must_use]
    pub const fn is_near_zero(self) -> bool {
        const EPS: f64 = 1e-8;
        self.x.abs() < EPS && self.y.abs() < EPS && self.z.abs() < EPS
    }

    #[inline]
    #[must_use]
    pub fn reflect(self, other: Self) -> Self {
        self - other * 2. * self.dot(other)
    }

    #[inline]
    #[must_use]
    pub fn refract(self, other: Self, etai_over_etat: f64) -> Self {
        let cos_theta = self.dot(-other).min(1.);
        let r_out_perp = (self + other * cos_theta) * etai_over_etat;
        let r_out_parallel = other * (-(1. - r_out_perp.square_length()).sqrt());
        r_out_perp + r_out_parallel
    }

    #[inline]
    #[must_use]
    pub fn inverse(self) -> Self {
        Self::new_array(self.to_array().map(f64::recip))
    }

    #[inline]
    #[must_use]
    pub const fn to_point(self) -> Point3 {
        self
    }

    #[inline]
    #[must_use]
    pub const fn component_mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    #[inline]
    #[must_use]
    pub const fn component_div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_array([self.x + rhs.x, self.y + rhs.y, self.z + rhs.z])
    }
}
impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new_array([self.x - rhs.x, self.y - rhs.y, self.z - rhs.z])
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self::new_array([-self.x, -self.y, -self.z])
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vec3) {
        *self = *self * rhs;
    }
}

impl Mul for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.component_mul(rhs)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self::new_array([self.x * rhs, self.y * rhs, self.z * rhs])
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self::new_array([self.x / rhs, self.y / rhs, self.z / rhs])
    }
}

impl Div for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Vec3) -> Self::Output {
        self.component_div(rhs)
    }
}

impl DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Vec3) {
        *self = *self / rhs;
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of range"),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of range"),
        }
    }
}

impl Bounded for Vec3 {
    fn get_aabbox(&self) -> AABBox {
        AABBox::from(*self)
    }

    fn get_surface_area(&self) -> f64 {
        0.
    }
}

impl From<[f64; 3]> for Vec3 {
    fn from(value: [f64; 3]) -> Self {
        Self::new_array(value)
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vec3::default(), |accum, other| accum + other)
    }
}

#[cfg(test)]
mod tests {
    use super::Vec3;
    use crate::test_utils::{assert_close, assert_vec};

    const TOLERANCE: f64 = 1e-12;

    #[test]
    fn arithmetic_and_vector_identities() {
        let a = Vec3::new(1., 2., 3.);
        let b = Vec3::new(-4., 5., 0.5);
        assert_vec(a + b, Vec3::new(-3., 7., 3.5), TOLERANCE);
        assert_vec(a - b, Vec3::new(5., -3., 2.5), TOLERANCE);
        assert_vec(a + Vec3::zero(), a, TOLERANCE);
        assert_vec(a - a, Vec3::zero(), TOLERANCE);
        assert_vec(a * 1., a, TOLERANCE);
        assert_vec(
            Vec3::new(1., -2., 4.) * 2.,
            Vec3::new(2., -4., 8.),
            TOLERANCE,
        );
        assert_vec(
            Vec3::new(1., -2., 4.) / 2.,
            Vec3::new(0.5, -1., 2.),
            TOLERANCE,
        );
        assert_vec(
            Vec3::new(2., 3., 4.) * Vec3::new(5., 2., -1.),
            Vec3::new(10., 6., -4.),
            TOLERANCE,
        );
        assert_vec(
            Vec3::new(10., 6., -4.) / Vec3::new(5., 2., -1.),
            Vec3::new(2., 3., 4.),
            TOLERANCE,
        );
        assert_close(a.dot(b), b.dot(a), TOLERANCE);
        assert_close(a.dot(a), a.square_length(), TOLERANCE);
        assert_vec(a.cross(b), -b.cross(a), TOLERANCE);
    }

    #[test]
    fn cross_products_follow_the_right_handed_basis() {
        let x = Vec3::new(1., 0., 0.);
        let y = Vec3::new(0., 1., 0.);
        let z = Vec3::new(0., 0., 1.);
        for (left, right, expected) in [(x, y, z), (y, z, x), (z, x, y)] {
            let cross = left.cross(right);
            assert_vec(cross, expected, TOLERANCE);
            assert_close(cross.dot(left), 0., TOLERANCE);
            assert_close(cross.dot(right), 0., TOLERANCE);
        }
    }

    #[test]
    fn length_normalization_reflection_and_refraction() {
        let vector = Vec3::new(1., 2., 2.);
        assert_close(vector.length(), 3., TOLERANCE);
        assert_close(vector.normalize().length(), 1., TOLERANCE);
        assert_vec(vector.normalize() * vector.length(), vector, TOLERANCE);
        assert_vec(
            Vec3::new(1., -1., 0.).reflect(Vec3::new(0., 1., 0.)),
            Vec3::new(1., 1., 0.),
            TOLERANCE,
        );
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

    #[test]
    fn constructors_indexing_and_sum_are_consistent() {
        let mut vector = Vec3::from([1., 2., 3.]);
        assert_eq!(vector.to_array(), [1., 2., 3.]);
        assert_eq!([vector[0], vector[1], vector[2]], [1., 2., 3.]);
        vector[1] = 4.;
        assert_vec(vector, Vec3::new(1., 4., 3.), TOLERANCE);
        assert_vec(std::iter::empty().sum(), Vec3::zero(), TOLERANCE);
        assert_vec(
            [Vec3::new(1., 2., 3.), Vec3::new(-1., 4., 2.)]
                .into_iter()
                .sum(),
            Vec3::new(0., 6., 5.),
            TOLERANCE,
        );
    }

    #[test]
    fn near_zero_uses_the_documented_component_threshold() {
        assert!(Vec3::new(0.99e-8, -0.99e-8, 0.).is_near_zero());
        assert!(!Vec3::new(1.01e-8, 0., 0.).is_near_zero());
    }
}
