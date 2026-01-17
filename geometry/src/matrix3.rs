use std::ops::{Add, Mul};

use super::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Matrix3(pub(crate) [[f64; 3]; 3]);

impl Matrix3 {
    // We allow this for specific math functions which normally use a lot of single character names
    #[allow(clippy::many_single_char_names)]
    #[must_use]
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

    // We allow this for specific math functions which normally use a lot of single character names
    #[allow(clippy::many_single_char_names)]
    #[must_use]
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
        out.iter_mut()
            .zip(self.0)
            .zip(rhs.0)
            .for_each(|((row_out, row_self), row_rhs)| {
                *row_out = (Vec3::from(row_self) + Vec3::from(row_rhs)).to_array();
            });
        Self(out)
    }
}

impl Mul for Matrix3 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut out: [[f64; 3]; 3] = Default::default();
        // It seems that transposing the rhs and then using iterators gives better assembly ¯\_(ツ)_/¯
        let mut rhs = rhs.0.map(std::iter::IntoIterator::into_iter);
        let rhs_transposed = Self(std::array::from_fn(|_| {
            std::array::from_fn(|i| rhs[i].next().unwrap())
        }));
        out.iter_mut().zip(self.0).for_each(|(row_out, row_self)| {
            row_out
                .iter_mut()
                .zip(rhs_transposed.0)
                .for_each(|(val, col_rhs)| *val = Vec3::from(row_self).dot(col_rhs.into()));
        });

        Self(out)
    }
}

impl Mul<Vec3> for Matrix3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        let mut out = Vec3::default().to_array();
        out.iter_mut().zip(self.0).for_each(|(out, row)| {
            *out = Vec3::from(row).dot(rhs);
        });

        Vec3::from(out)
    }
}
