#[cfg(feature = "euclid")]
use crate::aabbox::Box3DExt as _;
use crate::{aabbox::AABBox, bounded::Bounded, transformations::private::Token};
#[cfg(feature = "euclid")]
use euclid::UnknownUnit;
#[cfg(feature = "euclid")]
pub use inner::rotation;

#[cfg(feature = "euclid")]
mod inner {
    use euclid::Rotation3D;

    use crate::{aaplane::Axis, transformations::Transformation};

    #[must_use]
    pub fn rotation(angle: f64, axis: Axis) -> Transformation {
        let radians = angle.to_radians();
        match axis {
            Axis::X => Rotation3D::around_x(euclid::Angle { radians }),
            Axis::Y => Rotation3D::around_y(euclid::Angle { radians }),
            Axis::Z => Rotation3D::around_z(euclid::Angle { radians }),
        }
        .to_transform()
    }
}

#[cfg(not(feature = "euclid"))]
mod inner {
    use crate::{
        aaplane::Axis,
        matrix3::Matrix3,
        vec3::{Point3, Vec3},
    };

    #[must_use]
    pub fn rotation(angle: f64, axis: Axis) -> Transformation {
        let angle = angle.to_radians();
        match axis {
            Axis::X => [
                [1., 0., 0.],
                [0., angle.cos(), -angle.sin()],
                [0., angle.sin(), angle.cos()],
            ],
            Axis::Y => [
                [angle.cos(), 0., angle.sin()],
                [0., 1., 0.],
                [-angle.sin(), 0., angle.cos()],
            ],
            Axis::Z => [
                [angle.cos(), -angle.sin(), 0.],
                [angle.sin(), angle.cos(), 0.],
                [0., 0., 1.],
            ],
        }
        .into()
    }

    impl From<[[f64; 3]; 3]> for Transformation {
        fn from(value: [[f64; 3]; 3]) -> Self {
            Self {
                rotation: value.into(),
                ..Default::default()
            }
        }
    }

    impl From<Matrix3> for Transformation {
        fn from(value: Matrix3) -> Self {
            Self {
                rotation: value,
                ..Default::default()
            }
        }
    }

    impl From<Vec3> for Transformation {
        fn from(value: Vec3) -> Self {
            Self {
                translation: value,
                ..Default::default()
            }
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub struct Transformation {
        pub rotation: Matrix3,
        pub translation: Vec3,
    }

    impl Transformation {
        #[must_use]
        pub fn apply(self, transformation: Self) -> Self {
            Self {
                rotation: transformation.rotation * self.rotation,
                translation: transformation.translation
                    + transformation.rotation * self.translation,
            }
        }

        #[must_use]
        pub fn then(self, transformation: &Self) -> Self {
            self.apply(*transformation)
        }

        #[must_use]
        pub fn transform_point3d(self, point: Point3) -> Option<Point3> {
            Some(self.rotation * point + self.translation)
        }

        #[must_use]
        pub fn transform_vector3d(self, vec: Vec3) -> Vec3 {
            self.rotation * vec
        }

        #[must_use]
        pub fn inverse(self) -> Option<Self> {
            let rotation = self.rotation.inverse()?;
            Self {
                rotation,
                translation: -(rotation * self.translation),
            }
            .into()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{matrix3::Matrix3, transformations::Transformation, vec3::Vec3};

        #[test]
        fn inverse_times_itself_is_identity() {
            let mat = Matrix3::from([[2., -1., 1.], [1., 1., 1.], [1., 1., 2.]]);
            let translation = Vec3::new(0.5, 2., -1.);
            dbg!(mat.det());
            let trans = Transformation::from(mat);
            let trans = trans.apply(translation.into());
            let inv = trans.inverse().unwrap();
            let id = trans.apply(inv);
            for i in 0..3 {
                assert!((id.rotation.0[i][i] - 1.).abs() < f64::EPSILON);
                assert!(
                    (id.translation.inner()[i]).abs() < f64::EPSILON,
                    "id.translation.inner()[i] = {:?}",
                    id.translation.inner()[i]
                );
            }
        }

        #[test]
        fn inverse_of_identity_is_identity() {
            let id = Matrix3::from([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
            let trans_id = Transformation::from(id);
            let inv = trans_id.inverse().unwrap();
            for i in 0..3 {
                for j in 0..3 {
                    assert!((id.0[i][j] - inv.rotation.0[i][j]).abs() < f64::EPSILON);
                }
            }
        }
    }
}

#[cfg(feature = "euclid")]
pub type Transformation = euclid::Transform3D<f64, UnknownUnit, UnknownUnit>;

#[cfg(not(feature = "euclid"))]
pub use inner::{rotation, Transformation};

#[derive(Debug)]
pub struct Transformed<T> {
    transformation: Transformation,
    instance: T,
}

impl<T> Transformed<T> {
    pub const fn get_transformation(&self) -> Transformation {
        self.transformation
    }
    pub const fn get_instance(&self) -> &T {
        &self.instance
    }
}

impl<T> From<T> for Transformed<T> {
    fn from(value: T) -> Self {
        Self {
            transformation: Transformation::default(),
            instance: value,
        }
    }
}

mod private {
    pub struct Token {}
}

pub trait Transformable: Sized {
    fn transform<T: Into<Transformation>>(self, transformation: T) -> Transformed<Self> {
        self.transform_inner(transformation.into(), private::Token {})
    }

    #[doc(hidden)]
    fn transform_inner(self, transformation: Transformation, _: Token) -> Transformed<Self>;
}

impl<U> Transformable for U
where
    U: Into<Transformed<U>>,
{
    fn transform_inner(self, transformation: Transformation, _: Token) -> Transformed<Self> {
        self.into().transform_impl(transformation)
    }
}

impl<T> Transformed<T> {
    fn transform_impl(self, transformation: Transformation) -> Self {
        Self {
            transformation: self.transformation.then(&transformation),
            instance: self.instance,
        }
    }
}

impl<T> Bounded for Transformed<T>
where
    T: Bounded,
{
    fn get_aabbox(&self) -> AABBox {
        AABBox::from_points(
            self.get_instance()
                .get_aabbox()
                .get_points()
                .map(|p| self.transformation.transform_point3d(p).unwrap()),
        )
    }

    fn get_surface_area(&self) -> f64 {
        self.get_instance().get_surface_area()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "euclid")]
    use crate::aabbox::Box3DExt as _;
    use crate::{
        aabbox::AABBox,
        aaplane::Axis,
        bounded::Bounded,
        test_utils::{assert_close, assert_point, assert_vec},
        transformations::{rotation, Transformable, Transformation},
        vec3::{Point3, Vec3},
    };

    const TOLERANCE: f64 = 1e-12;

    fn translation(offset: Vec3) -> Transformation {
        #[cfg(not(feature = "euclid"))]
        {
            offset.into()
        }
        #[cfg(feature = "euclid")]
        {
            Transformation::translation(offset.x, offset.y, offset.z)
        }
    }

    fn transform_point(transform: Transformation, point: Point3) -> Point3 {
        transform.transform_point3d(point).unwrap()
    }

    fn assert_bounds_with_tolerance(actual: &AABBox, min: [f64; 3], max: [f64; 3], tolerance: f64) {
        let actual_min = Point3::new(
            *actual.axis(Axis::X).start(),
            *actual.axis(Axis::Y).start(),
            *actual.axis(Axis::Z).start(),
        );
        let actual_max = Point3::new(
            *actual.axis(Axis::X).end(),
            *actual.axis(Axis::Y).end(),
            *actual.axis(Axis::Z).end(),
        );
        assert_point(actual_min, Point3::from(min), tolerance);
        assert_point(actual_max, Point3::from(max), tolerance);
    }

    fn axis_contains_with_tolerance(box3: &AABBox, axis: Axis, coordinate: f64) -> bool {
        let range = box3.axis(axis);
        *range.start() <= coordinate + TOLERANCE && *range.end() + TOLERANCE >= coordinate
    }

    #[test]
    fn quarter_turns_have_the_expected_axis_directions_and_preserve_length() {
        assert_point(
            transform_point(rotation(90., Axis::X), Point3::from([0., 1., 0.])),
            Point3::from([0., 0., 1.]),
            TOLERANCE,
        );
        assert_point(
            transform_point(rotation(90., Axis::Y), Point3::from([1., 0., 0.])),
            Point3::from([0., 0., -1.]),
            TOLERANCE,
        );
        assert_point(
            transform_point(rotation(90., Axis::Z), Point3::from([1., 0., 0.])),
            Point3::from([0., 1., 0.]),
            TOLERANCE,
        );
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            for angle in [0., 360.] {
                assert_point(
                    transform_point(rotation(angle, axis), Point3::from([2., -3., 4.])),
                    Point3::from([2., -3., 4.]),
                    TOLERANCE,
                );
            }
            let rotated = rotation(90., axis).transform_vector3d(Vec3::new(1., 2., 3.));
            assert_close(rotated.length(), Vec3::new(1., 2., 3.).length(), TOLERANCE);
        }
    }

    #[test]
    fn translation_changes_points_but_not_vectors() {
        let transform = translation(Vec3::new(3., -2., 5.));
        assert_point(
            transform_point(transform, Point3::from([1., 2., 3.])),
            Point3::from([4., 0., 8.]),
            TOLERANCE,
        );
        assert_vec(
            transform.transform_vector3d(Vec3::new(1., 2., 3.)),
            Vec3::new(1., 2., 3.),
            TOLERANCE,
        );
    }

    #[test]
    fn then_and_custom_apply_compose_in_observable_application_order() {
        let translate = translation(Vec3::new(1., 0., 0.));
        let rotate = rotation(90., Axis::Z);
        let composed = translate.then(&rotate);
        assert_point(
            transform_point(composed, Point3::from([1., 2., 3.])),
            Point3::from([-2., 2., 3.]),
            TOLERANCE,
        );
        let sequential = transform_point(
            rotate,
            transform_point(translate, Point3::from([1., 2., 3.])),
        );
        assert_point(
            transform_point(composed, Point3::from([1., 2., 3.])),
            sequential,
            TOLERANCE,
        );

        #[cfg(not(feature = "euclid"))]
        {
            let applied = translate.apply(rotate);
            assert_point(
                transform_point(applied, Point3::from([1., 2., 3.])),
                sequential,
                TOLERANCE,
            );
        }
    }

    #[test]
    fn inverse_round_trips_point_and_vector_actions_in_both_orders() {
        let transform = rotation(37., Axis::Y).then(&translation(Vec3::new(0.5, 2., -1.)));
        let inverse = transform
            .inverse()
            .expect("rotation and translation invert");
        let point = Point3::from([2., -3., 4.]);
        let vector = Vec3::new(-1., 5., 2.);
        assert_point(
            transform_point(inverse, transform_point(transform, point)),
            point,
            TOLERANCE,
        );
        assert_point(
            transform_point(transform, transform_point(inverse, point)),
            point,
            TOLERANCE,
        );
        assert_vec(
            inverse.transform_vector3d(transform.transform_vector3d(vector)),
            vector,
            TOLERANCE,
        );
        assert_vec(
            transform.transform_vector3d(inverse.transform_vector3d(vector)),
            vector,
            TOLERANCE,
        );
    }

    #[cfg(not(feature = "euclid"))]
    #[test]
    fn custom_inverse_checks_full_rotation_and_translation_actions() {
        use crate::matrix3::Matrix3;

        let transform =
            Transformation::from(Matrix3::from([[2., -1., 1.], [1., 1., 1.], [1., 1., 2.]]))
                .then(&translation(Vec3::new(0.5, 2., -1.)));
        let inverse = transform.inverse().unwrap();
        let point = Point3::from([3., -2., 1.]);
        let vector = Vec3::new(-1., 4., 2.);
        assert_point(
            transform_point(transform.then(&inverse), point),
            point,
            TOLERANCE,
        );
        assert_point(
            transform_point(inverse.then(&transform), point),
            point,
            TOLERANCE,
        );
        assert_vec(
            transform.then(&inverse).transform_vector3d(vector),
            vector,
            TOLERANCE,
        );
        assert_vec(
            inverse.then(&transform).transform_vector3d(vector),
            vector,
            TOLERANCE,
        );
    }

    #[test]
    fn singular_transforms_have_no_inverse() {
        #[cfg(not(feature = "euclid"))]
        let singular = Transformation::from(crate::matrix3::Matrix3::from([
            [1., 0., 0.],
            [0., 0., 0.],
            [0., 0., 1.],
        ]));
        #[cfg(feature = "euclid")]
        let singular = Transformation::scale(0., 1., 1.);
        assert!(singular.inverse().is_none());
    }

    #[test]
    fn transformed_bounds_enclose_every_corner_and_transformable_chaining_matches_composition() {
        let source = AABBox::new(Point3::from([1., 2., 3.]), Point3::from([4., 5., 6.]));
        let translate = translation(Vec3::new(2., -1., 3.));
        let rotate = rotation(90., Axis::Z);
        let composed = translate.then(&rotate);
        let transformed = source.transform(translate).transform(rotate);
        let transformed_bounds = transformed.get_aabbox();
        let explicit_bounds = source.transform(composed).get_aabbox();
        #[cfg(not(feature = "euclid"))]
        let bounds_tolerance = 1e-3;
        #[cfg(feature = "euclid")]
        let bounds_tolerance = TOLERANCE;
        assert_bounds_with_tolerance(
            &transformed_bounds,
            [-4., 3., 6.],
            [-1., 6., 9.],
            bounds_tolerance,
        );
        assert_bounds_with_tolerance(
            &explicit_bounds,
            [-4., 3., 6.],
            [-1., 6., 9.],
            bounds_tolerance,
        );

        for corner in source.get_points() {
            let transformed_corner = transform_point(composed, corner);
            assert!(axis_contains_with_tolerance(
                &transformed_bounds,
                Axis::X,
                transformed_corner.x,
            ));
            assert!(axis_contains_with_tolerance(
                &transformed_bounds,
                Axis::Y,
                transformed_corner.y,
            ));
            assert!(axis_contains_with_tolerance(
                &transformed_bounds,
                Axis::Z,
                transformed_corner.z,
            ));
        }
        assert_close(
            transformed.get_surface_area(),
            source.get_surface_area(),
            TOLERANCE,
        );
    }
}
