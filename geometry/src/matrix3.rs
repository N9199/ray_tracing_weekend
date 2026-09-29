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

#[cfg(test)]
mod tests {
    use super::Matrix3;
    use crate::{
        test_utils::{assert_close, assert_vec},
        vec3::Vec3,
    };

    const TOLERANCE: f64 = 1e-12;

    fn assert_matrix(actual: Matrix3, expected: Matrix3) {
        for row in 0..3 {
            for column in 0..3 {
                assert_close(actual.0[row][column], expected.0[row][column], TOLERANCE);
            }
        }
    }

    #[test]
    fn identity_and_matrix_vector_products_use_rows_as_dot_products() {
        let identity = Matrix3::default();
        let vector = Vec3::new(2., -3., 4.);
        assert_vec(identity * vector, vector, TOLERANCE);
        assert_matrix(identity * identity, identity);

        let rows = Matrix3::from([[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]]);
        assert_vec(
            rows * Vec3::new(1., 0., -1.),
            Vec3::new(-2., -2., -2.),
            TOLERANCE,
        );
    }

    #[test]
    fn matrix_addition_is_elementwise_and_commutative() {
        let a = Matrix3::from([[1., 2., 3.], [4., 5., 6.], [7., 8., 9.]]);
        let b = Matrix3::from([[-1., 4., 2.], [0.5, -2., 3.], [6., 1., -5.]]);
        assert_matrix(a + Matrix3::from([[0.; 3]; 3]), a);
        assert_matrix(a + b, b + a);
        assert_matrix(
            a + b,
            Matrix3::from([[0., 6., 5.], [4.5, 3., 9.], [13., 9., 4.]]),
        );
    }

    #[test]
    fn matrix_multiplication_matches_hand_computation_and_associativity() {
        let a = Matrix3::from([[1., 2., 3.], [0., 1., 4.], [5., 6., 0.]]);
        let b = Matrix3::from([[2., 0., 1.], [3., 1., 0.], [4., 2., 1.]]);
        let c = Matrix3::from([[1., -1., 2.], [2., 3., 0.], [-2., 1., 1.]]);
        assert_matrix(
            a * b,
            Matrix3::from([[20., 8., 4.], [19., 9., 4.], [28., 6., 5.]]),
        );
        assert_matrix(
            b * a,
            Matrix3::from([[7., 10., 6.], [3., 7., 13.], [9., 16., 20.]]),
        );
        assert_matrix((a * b) * c, a * (b * c));
        assert_matrix(a * Matrix3::default(), a);
        assert_matrix(Matrix3::default() * a, a);
    }

    #[test]
    fn determinant_and_inverse_cover_full_matrix_results() {
        let identity = Matrix3::default();
        let diagonal = Matrix3::from([[2., 0., 0.], [0., 3., 0.], [0., 0., 4.]]);
        assert_close(identity.det(), 1., TOLERANCE);
        assert_close(diagonal.det(), 24., TOLERANCE);
        let swapped = Matrix3::from([[0., 1., 0.], [1., 0., 0.], [0., 0., 1.]]);
        assert_close(swapped.det(), -1., TOLERANCE);
        assert_close(
            Matrix3::from([[1., 2., 3.], [1., 2., 3.], [7., 8., 9.]]).det(),
            0.,
            TOLERANCE,
        );

        let matrix = Matrix3::from([[1., 2., 3.], [0., 1., 4.], [5., 6., 0.]]);
        let inverse = Matrix3::from([[-24., 18., 5.], [20., -15., -4.], [-5., 4., 1.]]);
        assert_close(matrix.det(), 1., TOLERANCE);
        assert_matrix(matrix.inverse().unwrap(), inverse);
        assert_matrix(matrix * inverse, identity);
        assert_matrix(inverse * matrix, identity);
        assert_matrix(
            diagonal.inverse().unwrap(),
            Matrix3::from([[0.5, 0., 0.], [0., 1. / 3., 0.], [0., 0., 0.25]]),
        );
    }

    #[test]
    fn singular_and_near_singular_inverse_behavior_is_explicit() {
        let singular = Matrix3::from([[1., 2., 3.], [2., 4., 6.], [0., 1., 1.]]);
        assert_eq!(singular.det(), 0.);
        assert!(singular.inverse().is_none());
        assert!(Matrix3::from([[0.; 3]; 3]).inverse().is_none());
        assert!(
            Matrix3::from([[f64::INFINITY, 0., 0.], [0., 1., 0.], [0., 0., 1.],])
                .inverse()
                .is_none()
        );
        assert!(
            Matrix3::from([[f64::MIN_POSITIVE, 0., 0.], [0., 0.5, 0.], [0., 0., 0.5],])
                .inverse()
                .is_none()
        );

        let near_singular = Matrix3::from([[1., 0., 0.], [0., 1., 0.], [0., 0., 1e-12]]);
        let inverse = near_singular
            .inverse()
            .expect("normal determinant is invertible");
        assert!(inverse.0.iter().flatten().all(|value| value.is_finite()));
        assert_matrix(near_singular * inverse, Matrix3::default());
    }
}
