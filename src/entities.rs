mod plane;
mod sphere;
mod aabox {
    use std::ops::RangeInclusive;

    use crate::{entities::Bounded, ray::Ray};

    #[derive(Debug, Clone, Copy)]
    pub struct AABBox {
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_min: f64,
        z_max: f64,
    }

    impl AABBox {
        pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64, z_min: f64, z_max: f64) -> Self {
            Self {
                x_min,
                x_max,
                y_min,
                y_max,
                z_min,
                z_max,
            }
        }
        pub fn is_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
            let x_tmin = (self.x_min - r.get_origin().get_x()) / r.get_direction().get_x();
            let x_tmax = (self.x_max - r.get_origin().get_x()) / r.get_direction().get_x();
            let (x_tmin, x_tmax) = (x_tmin.min(x_tmax), x_tmin.max(x_tmax));
            let (tmin, tmax) = (x_tmin, x_tmax);
            let y_tmin = (self.y_min - r.get_origin().get_y()) / r.get_direction().get_y();
            let y_tmax = (self.y_max - r.get_origin().get_y()) / r.get_direction().get_y();
            let (y_tmin, y_tmax) = (y_tmin.min(y_tmax), y_tmin.max(y_tmax));
            if tmax < y_tmin || tmin > y_tmax {
                return false;
            }
            let (tmin, tmax) = (tmin.max(y_tmin), tmax.min(y_tmax));
            let z_tmin = (self.z_min - r.get_origin().get_z()) / r.get_direction().get_z();
            let z_tmax = (self.z_max - r.get_origin().get_z()) / r.get_direction().get_z();
            let (z_tmin, z_tmax) = (z_tmin.min(z_tmax), z_tmin.max(z_tmax));
            if tmax < z_tmin || tmin > z_tmax {
                return false;
            }
            let (tmin, tmax) = (tmin.max(z_tmin), tmax.min(z_tmax));
            // TODO check if this are all the cases
            range.contains(&tmin) || range.contains(&tmax)
        }
        pub fn enclose<T: Bounded>(&mut self, object: T) {
            self.enclose_aabbox(object.get_aabbox());
        }

        fn enclose_aabbox(&mut self, aabbox: AABBox) {
            let AABBox { x_min, x_max, y_min, y_max, z_min, z_max } = aabbox;
            self.x_min = self.x_min.min(x_min);
            self.x_max = self.x_max.max(x_max);
            self.y_min = self.y_min.min(y_min);
            self.y_max = self.y_max.max(y_max);
            self.z_min = self.z_min.min(z_min);
            self.z_max = self.z_max.max(z_max);
        }
    }
    impl Bounded for AABBox {
        fn get_aabbox(&self) -> AABBox {
            *self
        }
    }
}
pub use aabox::AABBox;
pub use plane::Plane;
pub use sphere::Sphere;

pub trait Bounded {
    fn get_aabbox(&self) -> AABBox;
}
