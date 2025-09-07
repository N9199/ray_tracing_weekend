use std::{cmp::Ordering, ops::RangeInclusive};

use crate::bounded::Bounded;
use crate::vec3::Point3;

use super::aaplane::{AAPlane, Axis};

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
