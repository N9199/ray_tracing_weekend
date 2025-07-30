// #![feature(explicit_tail_calls)]
pub mod camera;
pub mod entities;
pub mod geometry;
pub mod hittable;
pub mod hittable_collections;
pub mod material;
pub mod pdf;
pub mod perlin;
pub mod ray;
pub mod scenes;
pub mod texture;
pub mod utils;

#[cfg(test)]
mod test;
