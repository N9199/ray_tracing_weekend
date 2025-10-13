use crate::aabox::AABBox;
#[cfg(feature = "euclid")]
use crate::vec3::Vec3;

pub trait Bounded {
    fn get_aabbox(&self) -> AABBox;
    fn get_surface_area(&self) -> f64;
}

#[cfg(feature = "euclid")]
impl Bounded for Vec3 {
    fn get_aabbox(&self) -> AABBox {
        use euclid::{Point3D, UnknownUnit};

        AABBox::from_points([Point3D::<f64, UnknownUnit>::from(self.to_array())])
    }

    fn get_surface_area(&self) -> f64 {
        0.0
    }
}
