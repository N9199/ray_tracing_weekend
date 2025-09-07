use std::ops::RangeInclusive;

use geometry::transformations::Transformed;

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
        let inv = self.get_transformation().rotation.inverse()?;
        let origin = inv * (r.get_origin() - self.get_transformation().translation);
        let direction = inv * (r.get_direction());
        let offsetted_ray = Ray::new(origin, direction);
        self.get_instance()
            .hit(&offsetted_ray, range)
            .map(|mut rec| {
                *rec.get_mut_p() = self.get_transformation().rotation * rec.get_p()
                    + self.get_transformation().translation;
                rec
            })
    }
}

impl<T> BoundedHittable for Transformed<T> where T: BoundedHittable {}
