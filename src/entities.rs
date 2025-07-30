mod aabox;
mod cuboid;
mod plane;
mod quadrilateral;
mod sphere;
pub mod transformations;
mod aaplane {

    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum Axis {
        X = 0,
        Y = 1,
        Z = 2,
    }

    pub const fn get_axis() -> [Axis; 3] {
        [Axis::X, Axis::Y, Axis::Z]
    }

    impl From<u8> for Axis {
        fn from(mut value: u8) -> Self {
            value = value.min(2);
            unsafe { core::mem::transmute(value) }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct AAPlane {
        pub coord: f64,
        pub axis: Axis,
    }
}
pub use aabox::AABBox;
pub use aaplane::{get_axis, AAPlane, Axis};
pub use cuboid::Cuboid;
pub use plane::Plane;
pub use quadrilateral::Quad;
pub use sphere::Sphere;

#[cfg(debug_assertions)]
pub(crate) use aabox::AABOX_HIT_COUNTER;
#[cfg(debug_assertions)]
pub(crate) use plane::PLANE_HIT_COUNTER;
#[cfg(debug_assertions)]
pub(crate) use quadrilateral::QUAD_HIT_COUNTER;
#[cfg(debug_assertions)]
pub(crate) use sphere::SPHERE_HIT_COUNTER;

pub trait Bounded {
    fn get_aabbox(&self) -> AABBox;
    fn get_surface_area(&self) -> f64;
}
