mod aabox;
mod plane;
mod sphere;

mod aaplane {

    #[repr(u8)]
    #[derive(Debug, Clone, Copy)]
    pub enum Axis {
        X = 0,
        Y = 1,
        Z = 2,
    }

    pub fn get_axis() -> [Axis; 3] {
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
pub use plane::Plane;
pub use sphere::Sphere;

pub trait Bounded {
    fn get_aabbox(&self) -> AABBox;
    fn get_surface_area(&self) -> f64;
}
