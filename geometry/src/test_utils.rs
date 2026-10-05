use crate::vec3::{Point3, Vec3};

/// Asserts that two scalar values differ by no more than an absolute tolerance.
///
/// # Panics
/// Panics if the difference exceeds `tolerance`.
#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert_within(actual, expected, tolerance, "");
}

#[track_caller]
fn assert_within(actual: f64, expected: f64, allowance: f64, component: &str) {
    let equal_infinities = actual.is_infinite()
        && expected.is_infinite()
        && actual.is_sign_positive() == expected.is_sign_positive();
    assert!(
        equal_infinities || (actual - expected).abs() <= allowance,
        "{component}{actual} != {expected} (absolute tolerance {allowance})"
    );
}

/// Asserts that two vectors are component-wise within an absolute tolerance.
///
/// # Panics
/// Panics if any component differs by more than `tolerance`.
#[track_caller]
pub fn assert_vec(actual: Vec3, expected: Vec3, tolerance: f64) {
    assert_within(actual.x, expected.x, tolerance, "vector x: ");
    assert_within(actual.y, expected.y, tolerance, "vector y: ");
    assert_within(actual.z, expected.z, tolerance, "vector z: ");
}

/// Asserts that two points are component-wise within an absolute tolerance.
///
/// # Panics
/// Panics if any component differs by more than `tolerance`.
#[track_caller]
pub fn assert_point(actual: Point3, expected: Point3, tolerance: f64) {
    assert_within(actual.x, expected.x, tolerance, "point x: ");
    assert_within(actual.y, expected.y, tolerance, "point y: ");
    assert_within(actual.z, expected.z, tolerance, "point z: ");
}

/// Asserts that two scalars are within a scale-relative tolerance.
///
/// # Panics
/// Panics if the difference exceeds the scaled allowance.
#[track_caller]
pub fn assert_close_scaled(actual: f64, expected: f64, relative_tolerance: f64) {
    assert_within_scaled(actual, expected, relative_tolerance, "");
}

#[track_caller]
fn assert_within_scaled(actual: f64, expected: f64, relative_tolerance: f64, component: &str) {
    let allowance = relative_tolerance * actual.abs().max(expected.abs()).max(1.);
    assert_within(actual, expected, allowance, component);
}

/// Asserts that two vectors are component-wise within a scale-relative tolerance.
///
/// # Panics
/// Panics if any component differs by more than its scaled allowance.
#[track_caller]
pub fn assert_vec_scaled(actual: Vec3, expected: Vec3, relative_tolerance: f64) {
    assert_within_scaled(actual.x, expected.x, relative_tolerance, "vector x: ");
    assert_within_scaled(actual.y, expected.y, relative_tolerance, "vector y: ");
    assert_within_scaled(actual.z, expected.z, relative_tolerance, "vector z: ");
}

/// Asserts that two points are component-wise within a scale-relative tolerance.
///
/// # Panics
/// Panics if any component differs by more than its scaled allowance.
#[track_caller]
pub fn assert_point_scaled(actual: Point3, expected: Point3, relative_tolerance: f64) {
    assert_within_scaled(actual.x, expected.x, relative_tolerance, "point x: ");
    assert_within_scaled(actual.y, expected.y, relative_tolerance, "point y: ");
    assert_within_scaled(actual.z, expected.z, relative_tolerance, "point z: ");
}

#[cfg(test)]
mod tests {
    use super::{assert_close, assert_close_scaled, assert_point, assert_vec};
    use crate::vec3::{Point3, Vec3};

    #[test]
    fn assertions_accept_values_within_their_tolerance() {
        assert_close(1. + 5e-11, 1., 1e-10);
        assert_vec(Vec3::new(1., 2., 3.), Vec3::new(1., 2., 3.), 0.);
        assert_point(Point3::new(1., 2., 3.), Point3::new(1., 2., 3.), 0.);
        assert_close_scaled(10. + 5e-9, 10., 1e-9);
    }
}
