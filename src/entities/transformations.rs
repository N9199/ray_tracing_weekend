use std::ops::RangeInclusive;

use crate::{
    geometry::{matrix3::Matrix3, vec3::Vec3},
    hittable::{BoundedHittable, HitRecord, Hittable},
    ray::Ray,
};

use super::{AABBox, Axis, Bounded};

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

#[derive(Debug, Default)]
pub struct Transformation {
    rotation: Matrix3,
    translation: Vec3,
}
impl Transformation {
    fn apply(self, transformation: Self) -> Self {
        Self {
            rotation: transformation.rotation * self.rotation,
            translation: self.translation + transformation.translation,
        }
    }
}

#[derive(Debug)]
pub struct Transformed<T> {
    transformation: Transformation,
    instance: T,
}

impl<T> From<T> for Transformed<T> {
    fn from(value: T) -> Self {
        Self {
            transformation: Transformation::default(),
            instance: value,
        }
    }
}

#[allow(private_bounds, private_interfaces)]
pub fn transform<U: Into<Transformed<U>>, T: Into<Transformation>>(
    value: U,
    transformation: T,
) -> Transformed<U> {
    value.into().transform_impl(transformation.into())
}

impl<T> Transformed<T> {
    fn transform_impl(self, transformation: Transformation) -> Self {
        Self {
            transformation: self.transformation.apply(transformation),
            instance: self.instance,
        }
    }

    pub fn transform<U: Into<Transformation>>(self, transformation: U) -> Self {
        self.transform_impl(transformation.into())
    }
}

impl<T> Hittable for Transformed<T>
where
    T: Hittable,
{
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // For simplicity if there's no inverse just say it's not hit.
        let inv = self.transformation.rotation.inverse()?;
        let origin = inv * (r.get_origin() - self.transformation.translation);
        let direction = inv * (r.get_direction());
        let offsetted_ray = Ray::new(origin, direction);
        self.instance.hit(&offsetted_ray, range).map(|mut rec| {
            *rec.get_mut_p() =
                self.transformation.rotation * rec.get_p() + self.transformation.translation;
            rec
        })
    }
}

impl<T> Bounded for Transformed<T>
where
    T: Bounded,
{
    fn get_aabbox(&self) -> AABBox {
        self.instance
            .get_aabbox()
            .get_points()
            .map(|p| self.transformation.rotation * p + self.transformation.translation)
            .into_iter()
            .collect::<Option<AABBox>>()
            .unwrap()
    }

    fn get_surface_area(&self) -> f64 {
        self.instance.get_surface_area()
    }
}

impl<T> BoundedHittable for Transformed<T> where T: BoundedHittable {}
