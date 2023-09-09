use core::ops::{Add, AddAssign, Mul};

use super::vec::Vec3;

#[derive(Debug, Default, Clone, Copy)]
pub struct Colour(Vec3);

impl Colour {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Colour(Vec3::new(r, g, b))
    }

    #[inline]
    fn write_colour(
        f: &mut core::fmt::Formatter<'_>,
        colour: &Colour,
        samples_per_pixel: i32,
    ) -> core::fmt::Result {
        let r = colour.0.get_x();
        let g = colour.0.get_y();
        let b = colour.0.get_z();

        let scale = 1. / samples_per_pixel as f64;

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
}

impl From<Vec3> for Colour {
    #[inline]
    fn from(other: Vec3) -> Self {
        Self(other)
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
        Self(self.0 * rhs.0)
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
