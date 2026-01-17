#[cfg(feature = "euclid")]
use crate::aabox::Box3DExt as _;
use crate::{aabox::AABBox, bounded::Bounded, transformations::private::Token};
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
            self.rotation * vec + self.translation
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
pub use inner::{Transformation, rotation};

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
