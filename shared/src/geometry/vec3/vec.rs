use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::{
    iter::Sum,
    ops::{DivAssign, Index, IndexMut},
};

use crate::entities::{AABBox, Bounded};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
    }

    #[inline]
    pub const fn new_array(inner: [f64; 3]) -> Self {
        Self(inner)
    }

    #[inline]
    pub const fn inner(self) -> [f64; 3] {
        self.0
    }

    #[inline]
    pub const fn get_x(self) -> f64 {
        self.0[0]
    }
    #[inline]
    pub const fn get_y(self) -> f64 {
        self.0[1]
    }
    #[inline]
    pub const fn get_z(self) -> f64 {
        self.0[2]
    }

    #[inline]
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    #[inline]
    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f64 {
        self.0[0] * rhs.0[0] + self.0[1] * rhs.0[1] + self.0[2] * rhs.0[2]
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self([
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        ])
    }

    #[inline]
    pub fn unit_vec(self) -> Self {
        self / self.length()
    }

    #[inline]
    pub fn is_near_zero(self) -> bool {
        const EPS: f64 = 1e-8;
        self.0[0].abs() < EPS && self.0[1].abs() < EPS && self.0[2].abs() < EPS
    }

    #[inline]
    pub fn reflect(self, other: Self) -> Self {
        self - other * 2. * self.dot(other)
    }

    #[inline]
    pub fn refract(self, other: Self, etai_over_etat: f64) -> Self {
        let cos_theta = self.dot(-other).min(1.);
        let r_out_perp = (self + other * cos_theta) * etai_over_etat;
        let r_out_parallel = other * (-(1. - r_out_perp.length_squared()).sqrt());
        r_out_perp + r_out_parallel
    }

    #[inline]
    pub fn inverse(self) -> Self {
        Self(self.0.map(f64::recip))
    }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
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
        Self([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}

impl Neg for Vec3 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self([-self.0[0], -self.0[1], -self.0[2]])
    }
}

impl MulAssign<f64> for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        self.0[0] *= rhs;
        self.0[1] *= rhs;
        self.0[2] *= rhs;
    }
}

impl MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vec3) {
        self.0[0] *= rhs.0[0];
        self.0[1] *= rhs.0[1];
        self.0[2] *= rhs.0[2];
    }
}

impl Mul for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self([self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs])
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

impl Div for Vec3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Vec3) -> Self::Output {
        Self([
            self.0[0] / rhs.0[0],
            self.0[1] / rhs.0[1],
            self.0[2] / rhs.0[2],
        ])
    }
}

impl DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Vec3) {
        self.0[0] /= rhs.0[0];
        self.0[1] /= rhs.0[1];
        self.0[2] /= rhs.0[2];
    }
}

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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
        Self(value)
    }
}

impl Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Vec3::default(), |accum, other| accum + other)
    }
}
