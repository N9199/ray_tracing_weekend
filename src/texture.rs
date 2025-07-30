use std::{
    fmt::Debug,
    ops::{Add, Mul, Rem},
    sync::Arc,
};

use crate::{
    geometry::vec3::{Colour, Point3, Vec3},
    perlin::Perlin,
};

pub trait Texture: Debug + Sync + Send {
    fn get_colour(&self, u: f64, v: f64, point: Point3) -> Colour;
}

#[derive(Debug, Clone, Copy)]
pub struct SolidColour(pub Colour);

impl Texture for SolidColour {
    fn get_colour(&self, _u: f64, _v: f64, _point: Point3) -> Colour {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(even: Arc<dyn Texture>, odd: Arc<dyn Texture>, scale: f64) -> Self {
        Self {
            inv_scale: scale.recip(),
            even,
            odd,
        }
    }

    pub fn new_with_colours(even: Colour, odd: Colour, scale: f64) -> Self {
        let even = Arc::new(SolidColour(even));
        let odd = Arc::new(SolidColour(odd));
        Self::new(even, odd, scale)
    }
}

impl Texture for CheckerTexture {
    fn get_colour(&self, u: f64, v: f64, point: Point3) -> Colour {
        if (u.mul(self.inv_scale).floor() + v.mul(self.inv_scale).floor()).rem(2.) == 0. {
            self.even.get_colour(u, v, point)
        } else {
            self.odd.get_colour(u, v, point)
        }
    }
}

#[derive(Clone)]
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl Debug for NoiseTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NoiseTexture")
            .field("noise", &"Noise")
            .field("scale", &self.scale)
            .finish()
    }
}

impl Default for NoiseTexture {
    fn default() -> Self {
        Self {
            noise: Default::default(),
            scale: 1.0,
        }
    }
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn get_colour(&self, u: f64, v: f64, point: Point3) -> Colour {
        // Vec3::new(1., 1., 1.)
        //     .mul(0.5)
        //     .mul(1. + self.noise.noise(&point.mul(self.scale)))
        //     .into()
        Vec3::new(0.5, 0.5, 0.5)
            .mul(
                self.scale
                    .mul(point.get_z())
                    .add(self.noise.turb(point, 7).mul(10.))
                    .sin()
                    .add(1.),
            )
            .into()
    }
}
