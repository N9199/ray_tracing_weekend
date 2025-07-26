use std::ops::{Add, Mul};

use itertools::iproduct;

use super::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Matrix3([[f64; 3]; 3]);

impl Matrix3 {
    pub fn inverse(self) -> Option<Self> {
        let det = self.det();
        if !det.is_normal() {
            return None;
        }
        let [[a, b, c], [d, e, f], [g, h, i]] = self.0;
        let [[a, b, c], [d, e, f], [g, h, i]] = [
            [e * i - f * h, f * g - d * i, d * h - e * g],
            [c * h - b * i, a * i - c * g, b * g - a * h],
            [b * f - c * e, c * d - a * f, a * e - b * d],
        ];
        Some(Matrix3([
            [a / det, d / det, g / det],
            [b / det, e / det, h / det],
            [c / det, f / det, i / det],
        ]))
    }

    pub fn det(self) -> f64 {
        let [[a, b, c], [d, e, f], [g, h, i]] = self.0;
        a * (e * i - f * h) + b * (f * g - d * i) + c * (d * h - e * g)
    }
}

impl From<[[f64; 3]; 3]> for Matrix3 {
    fn from(value: [[f64; 3]; 3]) -> Self {
        Self(value)
    }
}

impl Default for Matrix3 {
    fn default() -> Self {
        let inner: [[f64; 3]; 3] = [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
        Self(inner)
    }
}

impl Add for Matrix3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out: [[f64; 3]; 3] = Default::default();
        for (i, j) in iproduct!(0..(self.0.len()), 0..(rhs.0.len())) {
            out[i][j] = self.0[i][j] + rhs.0[i][j];
        }
        Self(out)
    }
}

impl Mul for Matrix3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut out: [[f64; 3]; 3] = Default::default();
        for (i, j, k) in iproduct!(0..(self.0.len()), 0..(rhs.0.len()), 0..(self.0.len())) {
            out[i][k] += self.0[i][j] * rhs.0[j][k];
        }
        Self(out)
    }
}

impl Mul<Vec3> for Matrix3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut out: Vec3 = Default::default();
        for (i, j) in iproduct!(0..(self.0.len()), 0..(self.0.len())) {
            out[i] += self.0[i][j] * rhs[j];
        }

        out
    }
}
