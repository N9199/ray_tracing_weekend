use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::{
    iter::Sum,
    ops::{DivAssign, Index, IndexMut},
};

use crate::{aabox::AABBox, bounded::Bounded, vec3::Point3};

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
    pub fn is_near_zero(self) -> bool {
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
    pub fn component_mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }

    #[inline]
    #[must_use]
    pub fn component_div(self, other: Self) -> Self {
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
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vec3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Mul for Vec3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::new_array([self.x * rhs.x, self.y * rhs.y, self.z * rhs.z])
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
        Self::new_array([self.x / rhs.x, self.y / rhs.y, self.z / rhs.z])
    }
}

impl DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Vec3) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
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
