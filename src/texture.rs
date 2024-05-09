use std::{
    fmt::Debug,
    ops::{Mul, Rem},
    sync::Arc,
};

use crate::vec3::{Colour, Point3};

pub trait Texture: Debug + Sync + Send {
    fn get_colour(&self, u: f64, v: f64, point: &Point3) -> Colour;
}

#[derive(Debug, Clone, Copy)]
pub struct SolidColour(pub Colour);

impl Texture for SolidColour {
    fn get_colour(&self, u: f64, v: f64, point: &Point3) -> Colour {
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
    fn get_colour(&self, u: f64, v: f64, point: &Point3) -> Colour {
        if (u.mul(self.inv_scale).floor() + v.mul(self.inv_scale).floor()).rem(2.) == 0. {
            self.even.get_colour(u, v, point)
        } else {
            self.odd.get_colour(u, v, point)
        }
    }
}
