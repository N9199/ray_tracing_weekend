use std::ops::RangeInclusive;

use geometry::{
    prelude::{Point3, Vec3},
    transformations::Transformed,
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    ray::Ray,
};

impl<T> Hittable for Transformed<T>
where
    T: Hittable,
{
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // For simplicity if there's no inverse just say it's not hit.
        let inv = self.get_transformation().inverse()?;
        let origin = inv.transform_point3d(r.get_origin())?;
        let direction = inv.transform_vector3d(r.get_direction());
        let offsetted_ray = Ray::new(origin, direction);
        self.get_instance()
            .hit(&offsetted_ray, range)
            .map(|mut rec| {
                let transformation = self.get_transformation();
                *rec.get_mut_p() = transformation.transform_point3d(rec.get_p()).unwrap();
                *rec.get_mut_normal() = transformation
                    .transform_vector3d(rec.get_normal())
                    .normalize();
                rec
            })
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        let Some(inverse) = self.get_transformation().inverse() else {
            return 0.;
        };
        let Some(local_origin) = inverse.transform_point3d(origin) else {
            return 0.;
        };
        self.get_instance()
            .pdf_value(local_origin, inverse.transform_vector3d(direction))
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        let Some(inverse) = self.get_transformation().inverse() else {
            return Vec3::from([1., 0., 0.]);
        };
        let Some(local_origin) = inverse.transform_point3d(origin) else {
            return Vec3::from([1., 0., 0.]);
        };
        self.get_transformation()
            .transform_vector3d(self.get_instance().random(local_origin, rng))
    }
}

impl<T> BoundedHittable for Transformed<T> where T: BoundedHittable {}
