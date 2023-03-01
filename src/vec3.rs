mod vec;
mod colour;

pub use vec::{Vec3, UnitSphere, get_unit_vec, get_in_unit_sphere, get_in_unit_disk};
pub use colour::{Colour, SampledColour};
pub type Point3 = Vec3;

