use crate::geometry::vec3::{Point3, Vec3};

#[derive(Debug, Default, Clone)]
pub struct Ray {
    origin: Point3,
    direction: Vec3,
}

impl Ray {
    #[inline]
    pub const fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    #[inline]
    pub const fn get_origin(&self) -> Point3 {
        self.origin
    }

    #[inline]
    pub const fn get_direction(&self) -> Vec3 {
        self.direction
    }

    #[inline]
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}
