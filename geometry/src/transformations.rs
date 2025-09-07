use crate::{
    aabox::AABBox, aaplane::Axis, bounded::Bounded, matrix3::Matrix3,
    transformations::private::Token, vec3::Vec3,
};

pub fn rotation(angle: f64, axis: Axis) -> Matrix3 {
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
    pub fn apply(self, transformation: Self) -> Self {
        Self {
            rotation: transformation.rotation * self.rotation,
            translation: transformation.translation + transformation.rotation * self.translation,
        }
    }
}

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
            transformation: self.transformation.apply(transformation),
            instance: self.instance,
        }
    }
}

impl<T> Bounded for Transformed<T>
where
    T: Bounded,
{
    fn get_aabbox(&self) -> AABBox {
        self.get_instance()
            .get_aabbox()
            .get_points()
            .map(|p| self.get_transformation().rotation * p + self.get_transformation().translation)
            .into_iter()
            .collect::<Option<AABBox>>()
            .unwrap()
    }

    fn get_surface_area(&self) -> f64 {
        self.get_instance().get_surface_area()
    }
}
