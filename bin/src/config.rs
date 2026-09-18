use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    image: ConfigImage,
}

impl Config {
    pub fn get_image(&mut self) -> Option<Image> {
        self.image.get()
    }
}

#[derive(Debug, Clone)]
enum ConfigImage {
    Image(Image),
    PreImage(PreImage),
    NoImage,
}

impl ConfigImage {
    pub fn get(&mut self) -> Option<Image> {
        match *self {
            Self::Image(v) => Some(v),
            Self::PreImage(v) => {
                if let Some(v) = v.fix() {
                    *self = Self::Image(v);
                    Some(v)
                } else {
                    *self = Self::NoImage;
                    None
                }
            }
            Self::NoImage => None,
        }
    }
}

impl<'de> Deserialize<'de> for ConfigImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::PreImage(PreImage::deserialize(deserializer)?))
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct PreImage {
    aspect_ratio: Option<f64>,
    image_width: Option<u16>,
    image_height: Option<u16>,
    samples_per_pixel: u16,
    max_depth: u32,
}

impl PreImage {
    pub(self) fn fix(self) -> Option<Image> {
        let checked_dimension = |value: f64| {
            let rounded = value.round(); // matches CameraBuilder's rounding
            if !rounded.is_finite() || rounded < 1.0 || rounded > f64::from(u16::MAX) {
                return None;
            }
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            Some(rounded as _)
        };

        let Self {
            aspect_ratio,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
        } = self;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let (aspect_ratio, image_height, image_width) =
            match (aspect_ratio, image_height, image_width) {
                (None, None, _) | (None, _, None) | (_, None, None) => return None,
                (None, Some(image_height), Some(image_width)) => (
                    f64::from(image_width) / f64::from(image_height),
                    image_height,
                    image_width,
                ),
                (Some(aspect_ratio), None, Some(image_width)) => (
                    aspect_ratio,
                    checked_dimension(f64::from(image_width) / aspect_ratio)?,
                    image_width,
                ),
                (Some(aspect_ratio), Some(image_height), None) => (
                    aspect_ratio,
                    image_height,
                    checked_dimension(f64::from(image_height) * aspect_ratio)?,
                ),
                (Some(aspect_ratio), Some(image_height), Some(image_width)) => (image_height
                    == checked_dimension(f64::from(image_width) / aspect_ratio)?)
                .then_some((aspect_ratio, image_height, image_width))?,
            };
        (aspect_ratio.is_finite() && aspect_ratio > 0. && image_width > 0 && image_height > 0)
            .then_some(Image {
                aspect_ratio,
                width: image_width,
                height: image_height,
                samples_per_pixel,
                max_depth,
            })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub aspect_ratio: f64,
    pub width: u16,
    pub height: u16,
    pub samples_per_pixel: u16,
    pub max_depth: u32,
}

#[cfg(test)]
mod tests {
    use geometry::test_utils::assert_close;

    use super::{Config, Image};

    const DEFAULT_FIELDS: &str = "samples_per_pixel = 17\nmax_depth = 23\n";

    fn parse_config(image_fields: &str) -> Result<Config, toml::de::Error> {
        toml::from_str(&format!("[image]\n{image_fields}"))
    }

    fn assert_image(
        image: Image,
        aspect_ratio: f64,
        width: u16,
        height: u16,
        samples_per_pixel: u16,
        max_depth: u32,
    ) {
        assert_close(image.aspect_ratio, aspect_ratio, 1e-12);
        assert_eq!(image.width, width);
        assert_eq!(image.height, height);
        assert_eq!(image.samples_per_pixel, samples_per_pixel);
        assert_eq!(image.max_depth, max_depth);
    }

    #[test]
    fn toml_config_completes_dimensions_and_preserves_settings() {
        let cases = [
            ("image_width = 800\nimage_height = 800\n", 1.0, 800, 800),
            (
                "image_width = 320\nimage_height = 180\n",
                320.0 / 180.0,
                320,
                180,
            ),
            (
                "aspect_ratio = 1.7777777777777777\nimage_width = 320\n",
                16.0 / 9.0,
                320,
                180,
            ),
            (
                "aspect_ratio = 1.7777777777777777\nimage_height = 180\n",
                16.0 / 9.0,
                320,
                180,
            ),
            (
                "aspect_ratio = 1.7777777777777777\nimage_width = 100\nimage_height = 56\n",
                16.0 / 9.0,
                100,
                56,
            ),
        ];

        for (dimensions, aspect, width, height) in cases {
            let mut config = parse_config(&format!("{dimensions}{DEFAULT_FIELDS}")).unwrap();
            let image = config.get_image().expect("valid image config");
            assert_image(image, aspect, width, height, 17, 23);
        }
    }

    #[test]
    fn image_config_rejects_incomplete_or_inconsistent_dimensions() {
        let cases = [
            "aspect_ratio = 1.7777777777777777\nimage_width = 100\nimage_height = 57\n",
            "aspect_ratio = 1.7777777777777777\n",
            "image_width = 320\n",
            "image_height = 180\n",
            "",
        ];

        for dimensions in cases {
            let mut config = parse_config(&format!("{dimensions}{DEFAULT_FIELDS}")).unwrap();
            assert!(config.get_image().is_none(), "accepted {dimensions:?}");
        }
    }

    #[test]
    fn image_config_accepts_zero_sampling_settings_as_currently_implemented() {
        for fields in [
            "image_width = 10\nimage_height = 10\nsamples_per_pixel = 0\nmax_depth = 23\n",
            "image_width = 10\nimage_height = 10\nsamples_per_pixel = 17\nmax_depth = 0\n",
        ] {
            let mut config = parse_config(fields).unwrap();
            let image = config
                .get_image()
                .expect("zero setting is currently accepted");
            assert_image(
                image,
                1.0,
                10,
                10,
                if fields.contains("samples_per_pixel = 0") {
                    0
                } else {
                    17
                },
                if fields.contains("max_depth = 0") {
                    0
                } else {
                    23
                },
            );
        }
    }

    #[test]
    fn inferred_dimensions_check_rounded_u16_boundaries() {
        let cases = [
            ("aspect_ratio = 1.0\nimage_width = 1\n", true),
            ("aspect_ratio = 3.0\nimage_width = 1\n", false),
            ("aspect_ratio = 1.0\nimage_width = 65535\n", true),
            ("aspect_ratio = 0.9999\nimage_width = 65535\n", false),
            ("aspect_ratio = 1.0\nimage_height = 1\n", true),
            ("aspect_ratio = 0.4\nimage_height = 1\n", false),
            ("aspect_ratio = 1.0\nimage_height = 65535\n", true),
            ("aspect_ratio = 1.01\nimage_height = 65535\n", false),
        ];

        for (dimensions, accepted) in cases {
            let mut config = parse_config(&format!("{dimensions}{DEFAULT_FIELDS}")).unwrap();
            assert_eq!(config.get_image().is_some(), accepted, "{dimensions:?}");
        }
    }

    #[test]
    fn invalid_aspect_ratios_are_rejected_after_toml_deserialization() {
        for ratio in ["nan", "inf", "-inf", "-1.0", "0.0"] {
            let parsed = parse_config(&format!(
                "aspect_ratio = {ratio}\nimage_width = 10\n{DEFAULT_FIELDS}"
            ));
            match parsed {
                Ok(mut config) => assert!(config.get_image().is_none(), "accepted {ratio}"),
                Err(error) => panic!("TOML rejected {ratio} before image validation: {error}"),
            }
        }

        for (ratio, width) in [("0.000001", "65535"), ("3.0", "1")] {
            let mut config = parse_config(&format!(
                "aspect_ratio = {ratio}\nimage_width = {width}\n{DEFAULT_FIELDS}"
            ))
            .unwrap();
            assert!(config.get_image().is_none());
        }
    }

    #[test]
    fn explicit_zero_dimensions_are_rejected() {
        for dimensions in [
            "image_width = 0\nimage_height = 10\n",
            "image_width = 10\nimage_height = 0\n",
        ] {
            let mut config = parse_config(&format!("{dimensions}{DEFAULT_FIELDS}")).unwrap();
            assert!(config.get_image().is_none());
        }
    }

    #[test]
    fn malformed_toml_config_shapes_fail_deserialization() {
        for document in [
            "",
            "[image]\nimage_width = 10\nimage_height = 10\nmax_depth = 2\n",
            "[image]\nimage_width = 10\nimage_height = 10\nsamples_per_pixel = 2\n",
            "[image]\naspect_ratio = true\nimage_width = 10\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = 2\n",
            "[image]\nimage_width = true\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = 2\n",
            "[image]\nimage_width = 10\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = true\n",
            "[image]\nimage_width = -1\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = 2\n",
            "[image]\nimage_width = 65536\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = 2\n",
            "[image]\nimage_width = 10\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = -1\n",
            "[image]\nimage_width = 10\nimage_height = 10\nsamples_per_pixel = 2\nmax_depth = 4294967296\n",
            "image = 3\n",
        ] {
            assert!(
                toml::from_str::<Config>(document).is_err(),
                "parsed {document:?}"
            );
        }
    }

    #[test]
    fn get_image_returns_stable_results_after_caching_valid_or_invalid_state() {
        let mut valid = parse_config(&format!(
            "image_width = 320\nimage_height = 180\n{DEFAULT_FIELDS}"
        ))
        .unwrap();
        let first = valid.get_image().expect("valid image");
        let second = valid.get_image().expect("cached valid image");
        assert_image(first, 320.0 / 180.0, 320, 180, 17, 23);
        assert_image(
            second,
            first.aspect_ratio,
            first.width,
            first.height,
            first.samples_per_pixel,
            first.max_depth,
        );

        let mut invalid = parse_config(&format!("image_width = 320\n{DEFAULT_FIELDS}")).unwrap();
        assert!(invalid.get_image().is_none());
        assert!(invalid.get_image().is_none());
    }
}
