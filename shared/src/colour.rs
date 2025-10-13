use core::ops::{Add, AddAssign, Mul};
use std::ops::{Div, DivAssign, MulAssign};

use geometry::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct Colour(Vec3);

impl Colour {
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Colour(Vec3::new(r, g, b))
    }

    #[inline]
    fn write_colour(
        f: &mut core::fmt::Formatter<'_>,
        colour: &Colour,
        samples_per_pixel: i32,
    ) -> core::fmt::Result {
        let r = colour.0.x;
        let g = colour.0.y;
        let b = colour.0.z;

        let scale = (samples_per_pixel as f64).recip();

        let r = (r * scale).sqrt();
        let g = (g * scale).sqrt();
        let b = (b * scale).sqrt();

        f.write_fmt(format_args!(
            "{} {} {}",
            (256. * r.clamp(0., 1.)) as u8,
            (256. * g.clamp(0., 1.)) as u8,
            (256. * b.clamp(0., 1.)) as u8
        ))
    }

    pub const fn into_inner(self) -> Vec3 {
        self.0
    }

    pub const fn from_vec3(vec: Vec3) -> Self {
        Self(vec)
    }

    pub fn from_array(inner: [f64; 3]) -> Self {
        Self(Vec3::from(inner))
    }

    // TODO: When const closure are stable and when Fn traits are "constified" make this const
    // See https://github.com/rust-lang/rust/issues/106003 and https://github.com/rust-lang/rust/issues/143874
    pub fn fix_nan(self) -> Self {
        Colour::from_array(
            self.into_inner()
                .to_array()
                .map(|v| if v.is_nan() { 0.0 } else { v }),
        )
    }
}

impl From<Vec3> for Colour {
    #[inline]
    fn from(other: Vec3) -> Self {
        Self(other)
    }
}

impl From<[f64; 3]> for Colour {
    #[inline]
    fn from(value: [f64; 3]) -> Self {
        Self(Vec3::from(value))
    }
}

impl AddAssign for Colour {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Add for Colour {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Mul<f64> for Colour {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl Mul for Colour {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.component_mul(rhs.0))
    }
}

impl MulAssign for Colour {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.component_mul(rhs.0);
    }
}

impl Div<f64> for Colour {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl Div for Colour {
    type Output = Self;

    fn div(self, rhs: Colour) -> Self::Output {
        Self(self.0.component_div(rhs.0))
    }
}

impl DivAssign for Colour {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.component_div(rhs.0);
    }
}

pub struct SampledColour(Colour, i32);

impl From<(Colour, i32)> for SampledColour {
    fn from((c, s): (Colour, i32)) -> Self {
        SampledColour(c, s)
    }
}

impl core::fmt::Display for SampledColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Colour::write_colour(f, &self.0, self.1)
    }
}
