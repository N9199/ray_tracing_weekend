use crate::aabox::AABBox;

pub trait Bounded {
    fn get_aabbox(&self) -> AABBox;
    fn get_surface_area(&self) -> f64;
}
