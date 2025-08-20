#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{cmp::Ordering, ops::RangeInclusive};

use crate::{entities::Bounded, geometry::vec3::Point3, ray::Ray};

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

impl FromIterator<Point3> for Option<AABBox> {
    fn from_iter<T: IntoIterator<Item = Point3>>(iter: T) -> Self {
        iter.into_iter().fold(None, |accum, item| match accum {
            Some(aabox) => Some(aabox.enclose(&item)),
            None => Some(item.into()),
        })
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static AABOX_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl AABBox {
    pub const fn new(
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        z_min: f64,
        z_max: f64,
    ) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
            z_min,
            z_max,
        }
    }

    pub const fn get_points(self) -> [Point3; 8] {
        [
            Point3::new(self.x_min, self.y_min, self.z_min),
            Point3::new(self.x_min, self.y_max, self.z_min),
            Point3::new(self.x_min, self.y_min, self.z_max),
            Point3::new(self.x_min, self.y_max, self.z_max),
            Point3::new(self.x_max, self.y_min, self.z_min),
            Point3::new(self.x_max, self.y_max, self.z_min),
            Point3::new(self.x_max, self.y_min, self.z_max),
            Point3::new(self.x_max, self.y_max, self.z_max),
        ]
    }

    // TODO: When float operations are const, constify this function
    fn pad_to_minimum(mut self) -> Self {
        let dx = self.axis(Axis::X).end() - self.axis(Axis::X).start();
        let dy = self.axis(Axis::Y).end() - self.axis(Axis::Y).start();
        let dz = self.axis(Axis::Z).end() - self.axis(Axis::Z).start();
        const DELTA: f64 = 0.0001;
        if dx < DELTA {
            self.x_min -= DELTA;
            self.x_max += DELTA;
        }
        if dy < DELTA {
            self.y_min -= DELTA;
            self.y_max += DELTA;
        }
        if dz < DELTA {
            self.z_min -= DELTA;
            self.z_max += DELTA;
        }
        self
    }

    #[inline]
    pub const fn axis(&self, axis: Axis) -> RangeInclusive<f64> {
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
        self.hit(r, range).is_some()
    }

    #[inline]
    pub fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<f64> {
        // // dbg!(r);
        let x_tmin = (self.x_min - r.get_origin().get_x()) / r.get_direction().get_x();
        let x_tmax = (self.x_max - r.get_origin().get_x()) / r.get_direction().get_x();
        let (x_tmin, x_tmax) = if r.get_direction().get_x().is_sign_negative() {
            (x_tmax, x_tmin)
        } else {
            (x_tmin, x_tmax)
        };
        let (tmin, tmax) = (x_tmin, x_tmax);
        let y_tmin = (self.y_min - r.get_origin().get_y()) / r.get_direction().get_y();
        let y_tmax = (self.y_max - r.get_origin().get_y()) / r.get_direction().get_y();
        let (y_tmin, y_tmax) = if r.get_direction().get_y().is_sign_negative() {
            (y_tmax, y_tmin)
        } else {
            (y_tmin, y_tmax)
        };
        // // dbg!(tmax, tmin, y_tmin, y_tmax);
        if tmax < y_tmin || tmin > y_tmax {
            return None;
        }
        let (tmin, tmax) = (tmin.max(y_tmin), tmax.min(y_tmax));
        let z_tmin = (self.z_min - r.get_origin().get_z()) / r.get_direction().get_z();
        let z_tmax = (self.z_max - r.get_origin().get_z()) / r.get_direction().get_z();
        let (z_tmin, z_tmax) = if r.get_direction().get_z().is_sign_negative() {
            (z_tmax, z_tmin)
        } else {
            (z_tmin, z_tmax)
        };
        // // dbg!(tmax, tmin, z_tmin, z_tmax);
        if tmax < z_tmin || tmin > z_tmax {
            return None;
        }
        let (tmin, tmax) = (tmin.max(z_tmin), tmax.min(z_tmax));
        // TODO check if this are all the cases
        let out = range.start().max(tmin) <= range.end().min(tmax);

        #[cfg(feature = "hit_counters")]
        if out {
            // dbg!("AABox Hit");
            AABOX_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        }
        // dbg!(*self, r, tmin, tmax, &range, out);
        out.then_some(range.start().max(tmin))
    }

    pub fn enclose<T: Bounded>(mut self, object: &T) -> Self {
        self.enclose_aabbox(object.get_aabbox());
        self
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
        *self = self.pad_to_minimum();
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
    #[inline]
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

impl From<Point3> for AABBox {
    fn from(value: Point3) -> Self {
        Self {
            x_min: value.get_x(),
            x_max: value.get_x(),
            y_min: value.get_y(),
            y_max: value.get_y(),
            z_min: value.get_z(),
            z_max: value.get_z(),
        }
    }
}
