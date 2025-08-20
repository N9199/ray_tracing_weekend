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
        use ConfigImage::*;
        match *self {
            Image(v) => Some(v),
            PreImage(v) => match v.fix() {
                Some(v) => {
                    *self = Image(v);
                    Some(v)
                }
                None => {
                    *self = NoImage;
                    None
                }
            },
            NoImage => None,
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
    image_width: Option<u32>,
    image_height: Option<u32>,
    samples_per_pixel: u16,
    max_depth: u8,
}

impl PreImage {
    pub(self) fn fix(self) -> Option<Image> {
        let Self {
            aspect_ratio,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
        } = self;
        let (aspect_ratio, image_height, image_width) =
            match (aspect_ratio, image_height, image_width) {
                (None, None, _) | (None, _, None) | (_, None, None) => return None,
                (None, Some(image_height), Some(image_width)) => (
                    (image_width as f64) / (image_height as f64),
                    image_height,
                    image_width,
                ),
                (Some(aspect_ratio), None, Some(image_width)) => (
                    aspect_ratio,
                    (image_width as f64 / aspect_ratio) as _,
                    image_width,
                ),
                (Some(aspect_ratio), Some(image_height), None) => (
                    aspect_ratio,
                    image_height,
                    (image_height as f64 * aspect_ratio) as _,
                ),
                (Some(aspect_ratio), Some(image_height), Some(image_width)) => {
                    (aspect_ratio, image_height, image_width)
                }
            };
        Some(Image {
            aspect_ratio,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Image {
    pub aspect_ratio: f64,
    pub image_width: u32,
    pub image_height: u32,
    pub samples_per_pixel: u16,
    pub max_depth: u8,
}
