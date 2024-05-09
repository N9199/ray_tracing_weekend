use std::{cmp::Ordering, ops::RangeInclusive};

use crate::{entities::Bounded, ray::Ray};

use super::{AAPlane, Axis};

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

    #[inline]
    pub fn axis(&self, axis: Axis) -> RangeInclusive<f64> {
        match axis {
            Axis::X => self.x_min..=self.x_max,
            Axis::Y => self.y_min..=self.y_max,
            Axis::Z => self.z_min..=self.z_max,
        }
    }

    #[inline]
    pub fn is_hit2(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        for i in 0..3 {
            let inv_d = r.get_direction().inverse()[i];
            let origin = r.get_origin()[i];

            let mut t0 = (self.axis((i as u8).into()).start() - origin) * inv_d;
            let mut t1 = (self.axis((i as u8).into()).end() - origin) * inv_d;
            if inv_d < 0. {
                (t0, t1) = (t1, t0);
            }

            let t0 = t0.max(*range.start());
            let t1 = t1.min(*range.end());
            if t1 <= t0 {
                return false;
            }
        }
        true
    }

    #[inline]
    pub fn is_hit(&self, r: &Ray, range: RangeInclusive<f64>) -> bool {
        let x_tmin = (self.x_min - r.get_origin().get_x()) / r.get_direction().get_x();
        let x_tmax = (self.x_max - r.get_origin().get_x()) / r.get_direction().get_x();
        let (x_tmin, x_tmax) = if r.get_direction().get_x() < 0. {
            (x_tmax, x_tmin)
        } else {
            (x_tmin, x_tmax)
        };
        let (tmin, tmax) = (x_tmin, x_tmax);
        let y_tmin = (self.y_min - r.get_origin().get_y()) / r.get_direction().get_y();
        let y_tmax = (self.y_max - r.get_origin().get_y()) / r.get_direction().get_y();
        let (y_tmin, y_tmax) = if r.get_direction().get_y() < 0. {
            (y_tmax, y_tmin)
        } else {
            (y_tmin, y_tmax)
        };
        if tmax < y_tmin || tmin > y_tmax {
            return false;
        }
        let (tmin, tmax) = (tmin.max(y_tmin), tmax.min(y_tmax));
        let z_tmin = (self.z_min - r.get_origin().get_z()) / r.get_direction().get_z();
        let z_tmax = (self.z_max - r.get_origin().get_z()) / r.get_direction().get_z();
        let (z_tmin, z_tmax) = if r.get_direction().get_z() < 0. {
            (z_tmax, z_tmin)
        } else {
            (z_tmin, z_tmax)
        };
        if tmax < z_tmin || tmin > z_tmax {
            return false;
        }
        let (tmin, tmax) = (tmin.max(z_tmin), tmax.min(z_tmax));
        // TODO check if this are all the cases
        range.start().max(tmin) <= range.end().min(tmax)
    }

    pub fn enclose<T: Bounded>(&mut self, object: &T) {
        self.enclose_aabbox(object.get_aabbox());
    }

    fn enclose_aabbox(&mut self, aabbox: AABBox) {
        let AABBox {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        } = aabbox;
        self.x_min = self.x_min.min(x_min);
        self.x_max = self.x_max.max(x_max);
        self.y_min = self.y_min.min(y_min);
        self.y_max = self.y_max.max(y_max);
        self.z_min = self.z_min.min(z_min);
        self.z_max = self.z_max.max(z_max);
    }

    pub fn left_of(&self, plane: AAPlane) -> bool {
        self.axis(plane.axis as _).end() < &plane.coord
    }

    pub fn right_of(&self, plane: AAPlane) -> bool {
        self.axis(plane.axis as _).start() > &plane.coord
    }

    pub fn compare_by_axis(&self, other: &Self, axis: Axis) -> Ordering {
        self.axis(axis).start().total_cmp(other.axis(axis).start())
    }
}
impl Bounded for AABBox {
    fn get_aabbox(&self) -> AABBox {
        *self
    }

    fn get_surface_area(&self) -> f64 {
        let dx = self.x_max - self.x_min;
        let dy = self.y_max - self.y_min;
        let dz = self.z_max - self.z_min;
        2. * (dx * dy + dx * dz + dy * dz)
    }
}
