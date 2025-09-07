mod cuboid;
mod plane;
mod quadrilateral;
mod sphere;
pub mod transformations;
pub use cuboid::Cuboid;
pub use plane::Plane;
pub use quadrilateral::Quad;
pub use sphere::Sphere;

// #[cfg(feature = "hit_counters")]
// pub(crate) use aabox::AABOX_HIT_COUNTER;

#[cfg(feature = "hit_counters")]
pub(crate) use plane::PLANE_HIT_COUNTER;

#[cfg(feature = "hit_counters")]
pub(crate) use quadrilateral::QUAD_HIT_COUNTER;

#[cfg(feature = "hit_counters")]
pub(crate) use sphere::SPHERE_HIT_COUNTER;
