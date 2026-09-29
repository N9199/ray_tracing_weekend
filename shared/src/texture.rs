use std::{
    fmt::Debug,
    ops::{Add, Mul, Rem},
    sync::Arc,
};

use geometry::vec3::{Point3, Vec3};

use crate::{colour::Colour, perlin::Perlin};

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
    #[must_use]
    pub fn new(even: Arc<dyn Texture>, odd: Arc<dyn Texture>, scale: f64) -> Self {
        Self {
            inv_scale: scale.recip(),
            even,
            odd,
        }
    }

    #[must_use]
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
            noise: Perlin::default(),
            scale: 1.0,
        }
    }
}

impl NoiseTexture {
    #[must_use]
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn get_colour(&self, _u: f64, _v: f64, point: Point3) -> Colour {
        Vec3::new(0.5, 0.5, 0.5)
            .mul(
                self.scale
                    .mul(point.z)
                    .add(self.noise.turb(point, 7).mul(10.))
                    .sin()
                    .add(1.),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use geometry::{test_utils::assert_vec, vec3::Point3};

    use crate::{
        colour::Colour,
        texture::{CheckerTexture, NoiseTexture, SolidColour, Texture},
    };

    const TOLERANCE: f64 = 1e-12;

    fn assert_colour(actual: Colour, expected: Colour) {
        assert_vec(actual.into_inner(), expected.into_inner(), TOLERANCE);
    }

    #[test]
    fn solid_colour_ignores_coordinates_and_point() {
        let colour = Colour::new(0.15, 0.45, 0.9);
        let texture = SolidColour(colour);
        assert_colour(
            texture.get_colour(0.23, 0.71, Point3::new(4., -3., 9.)),
            colour,
        );
        assert_colour(
            texture.get_colour(-12., 5.5, Point3::new(-8., 1., 0.)),
            colour,
        );
    }

    #[test]
    fn checker_texture_uses_uv_cell_parity_including_negative_values_and_boundaries() {
        let even = Colour::new(0.1, 0.2, 0.3);
        let odd = Colour::new(0.8, 0.6, 0.4);
        let checker = CheckerTexture::new_with_colours(even, odd, 1.);
        let point = Point3::new(100., -200., 300.);
        for (u, v, expected) in [
            (0.2, 0.3, even),
            (1.2, 0.3, odd),
            (0.2, 1.3, odd),
            (1.2, 1.3, even),
            (-0.2, 0.3, odd),
            (1., 0., odd),
            (2., 0., even),
            (-1., 0., odd),
        ] {
            assert_colour(checker.get_colour(u, v, point), expected);
        }
        assert_colour(
            checker.get_colour(0.2, 0.3, Point3::new(-900., 400., 2.)),
            even,
        );
    }

    #[test]
    fn checker_colour_constructor_matches_solid_texture_constructor() {
        let even = Colour::new(0.25, 0.5, 0.75);
        let odd = Colour::new(0.9, 0.6, 0.3);
        let from_colours = CheckerTexture::new_with_colours(even, odd, 0.5);
        let from_textures =
            CheckerTexture::new(Arc::new(SolidColour(even)), Arc::new(SolidColour(odd)), 0.5);
        for (u, v) in [(0.1, 0.1), (0.7, 0.1), (0.1, 0.7), (-0.1, 0.1)] {
            let point = Point3::new(u, v, 8.);
            assert_colour(
                from_colours.get_colour(u, v, point),
                from_textures.get_colour(u, v, point),
            );
        }
    }

    #[test]
    fn noise_texture_is_repeatable_uv_independent_and_grayscale_in_range() {
        let texture = NoiseTexture::new(1.7);
        let point = Point3::new(-2.31, 4.17, 0.83);
        let first = texture.get_colour(0.1, 0.2, point);
        let repeated = texture.get_colour(0.1, 0.2, point);
        let changed_uv = texture.get_colour(-100., 25., point);
        assert_colour(first, repeated);
        assert_colour(first, changed_uv);

        let channels = first.into_inner();
        assert!(channels.x.is_finite() && channels.y.is_finite() && channels.z.is_finite());
        assert!((0. ..=1.).contains(&channels.x));
        assert!((0. ..=1.).contains(&channels.y));
        assert!((0. ..=1.).contains(&channels.z));
        assert_colour(first, Colour::new(channels.x, channels.x, channels.x));
    }
}
