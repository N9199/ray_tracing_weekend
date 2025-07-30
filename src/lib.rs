#![warn(clippy::missing_const_for_fn)]
#![allow(unused, dead_code)]
pub mod bvh;
pub mod camera;
pub mod entities;
pub mod geometry;
pub mod hittable;
pub mod hittable_list;
pub mod material;
pub mod pdf;
pub mod perlin;
pub mod ray;
mod scenes;
pub mod texture;
pub mod utils;

#[cfg(test)]
mod test;
