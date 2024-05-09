use rand::{distributions::Open01, prelude::Distribution};

use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};
use std::ops::Index;

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
            let out = Vec3::new(rng.sample(Open01), rng.sample(Open01), rng.sample(Open01));
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
            let out = Vec3::new(rng.sample(Open01), rng.sample(Open01), 0.);
            if out.length_squared() < 1. {
                return out;
            }
        }
    }
}

#[inline]
pub fn get_unit_vec<T: rand::Rng>(rng: &mut T) -> Vec3 {
    get_in_unit_sphere(rng).unit_vec()
}

#[inline]
pub fn get_in_unit_sphere<T: rand::Rng>(rng: &mut T) -> Vec3 {
    rng.sample(UnitSphere)
}

#[inline]
pub fn get_in_unit_disk<T: rand::Rng>(rng: &mut T) -> Vec3 {
    rng.sample(UnitDisk)
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Vec3([f64; 3]);

impl Vec3 {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self([x, y, z])
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

impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
