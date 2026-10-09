use std::{collections::BTreeMap, ops::RangeInclusive, sync::Arc};

use geometry::{
    aaplane::Axis,
    bounded::Bounded,
    transformations::{Transformable as _, Transformation, rotation},
    vec3::{Point3, Translation3, Vec3},
};
use serde::Deserialize;
use shared::{
    camera::CameraBuilder,
    colour::Colour,
    entities::{Cuboid, Plane, Quad, Sphere},
    hittable::{BoundedHittable, HitRecord, Hittable},
    hittable_collections::hittable_list::HittableList,
    material::{Dielectric, DiffuseLight, Lambertian, Material, Metal},
    ray::Ray,
    texture::{CheckerTexture, NoiseTexture, SolidColour, Texture},
};

use crate::Output;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneConfig {
    camera: CameraConfig,
    #[serde(default)]
    materials: BTreeMap<String, MaterialConfig>,
    #[serde(default)]
    objects: Vec<ObjectConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CameraConfig {
    look_from: [f64; 3],
    look_at: [f64; 3],
    #[serde(default = "default_vfov")]
    vfov: f64,
    #[serde(default = "default_background")]
    background: [f64; 3],
    #[serde(default = "default_vup")]
    vup: [f64; 3],
    focus_distance: Option<f64>,
    #[serde(default)]
    defocus_angle: f64,
}

const fn default_vfov() -> f64 {
    40.
}

const fn default_background() -> [f64; 3] {
    [0., 0., 0.]
}

const fn default_vup() -> [f64; 3] {
    [0., 1., 0.]
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum MaterialConfig {
    Lambertian {
        texture: TextureConfig,
    },
    Metal {
        albedo: [f64; 3],
        #[serde(default)]
        fuzz: f64,
    },
    Dielectric {
        index_of_refraction: f64,
    },
    DiffuseLight {
        texture: TextureConfig,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TextureConfig {
    Solid {
        color: [f64; 3],
    },
    Checker {
        even: [f64; 3],
        odd: [f64; 3],
        scale: f64,
    },
    Noise {
        scale: f64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectConfig {
    material: String,
    #[serde(default)]
    sample_as_light: bool,
    shape: ShapeConfig,
    #[serde(default)]
    transforms: Vec<TransformConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ShapeConfig {
    Sphere {
        center: [f64; 3],
        radius: f64,
    },
    Plane {
        point: [f64; 3],
        normal: [f64; 3],
    },
    Quad {
        point: [f64; 3],
        u: [f64; 3],
        v: [f64; 3],
    },
    Cuboid {
        min: [f64; 3],
        max: [f64; 3],
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TransformConfig {
    Translate { offset: [f64; 3] },
    Rotate { axis: RotationAxis, degrees: f64 },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RotationAxis {
    X,
    Y,
    Z,
}

impl RotationAxis {
    const fn geometry_axis(&self) -> Axis {
        match self {
            Self::X => Axis::X,
            Self::Y => Axis::Y,
            Self::Z => Axis::Z,
        }
    }
}

impl SceneConfig {
    /// Validates the scene and constructs the objects and camera used by the renderer.
    ///
    /// # Errors
    ///
    /// Returns an error when a material reference, camera setting, object, or transform is invalid.
    pub fn build_scene(&self) -> Result<Output, String> {
        self.validate()?;

        let materials = self
            .materials
            .iter()
            .map(|(name, config)| (name.clone(), config.build()))
            .collect::<BTreeMap<_, _>>();
        let mut world = HittableList::default();
        let mut lights = HittableList::default();

        for object in &self.objects {
            let material = Arc::clone(&materials[&object.material]);
            let transform = object.transformation();
            world.add(
                object
                    .shape
                    .build(Arc::clone(&material))
                    .transform(transform),
            );
            if object.sample_as_light {
                lights.add(object.shape.build(material).transform(transform));
            }
        }

        let look_from = point(self.camera.look_from);
        let look_at = point(self.camera.look_at);
        let focus_distance = self
            .camera
            .focus_distance
            .unwrap_or_else(|| (look_from - look_at).length());
        let camera = CameraBuilder::new()
            .with_lookfrom(look_from)
            .with_lookat(look_at)
            .with_vup(vector(self.camera.vup))
            .with_vfov(self.camera.vfov)
            .with_background(colour(self.camera.background))
            .with_focus_dist(focus_distance)
            .with_defocus_angle(self.camera.defocus_angle);

        Ok((Box::new(world), Box::new(lights), camera))
    }

    fn validate(&self) -> Result<(), String> {
        let camera = &self.camera;
        require_finite("camera.look_from", &camera.look_from)?;
        require_finite("camera.look_at", &camera.look_at)?;
        require_finite("camera.vup", &camera.vup)?;
        require_colour("camera.background", camera.background)?;
        if (point(camera.look_from) - point(camera.look_at)).length() <= f64::EPSILON {
            return Err("camera.look_from and camera.look_at must differ".into());
        }
        if vector(camera.vup).length() <= f64::EPSILON {
            return Err("camera.vup must be non-zero".into());
        }
        if !camera.vfov.is_finite() || !(0. ..180.).contains(&camera.vfov) {
            return Err("camera.vfov must be finite and between 0 and 180 degrees".into());
        }
        if camera
            .focus_distance
            .is_some_and(|value| !value.is_finite() || value <= 0.)
        {
            return Err("camera.focus_distance must be finite and positive".into());
        }
        if !camera.defocus_angle.is_finite() || camera.defocus_angle < 0. {
            return Err("camera.defocus_angle must be finite and non-negative".into());
        }
        if self.objects.is_empty() {
            return Err("scene.objects must contain at least one object".into());
        }

        for (name, material) in &self.materials {
            material.validate(&format!("materials.{name}"))?;
        }
        for (index, object) in self.objects.iter().enumerate() {
            let field = format!("objects[{index}]");
            if !self.materials.contains_key(&object.material) {
                return Err(format!(
                    "{field}.material references unknown material {:?}",
                    object.material
                ));
            }
            object.shape.validate(&format!("{field}.shape"))?;
            for (transform_index, transform) in object.transforms.iter().enumerate() {
                transform.validate(&format!("{field}.transforms[{transform_index}]"))?;
            }
            if object.sample_as_light {
                if !matches!(
                    object.shape,
                    ShapeConfig::Sphere { .. } | ShapeConfig::Quad { .. }
                ) {
                    return Err(format!(
                        "{field}.sample_as_light is supported only for sphere and quad shapes"
                    ));
                }
                if !matches!(
                    self.materials[&object.material],
                    MaterialConfig::DiffuseLight { .. }
                ) {
                    return Err(format!(
                        "{field}.sample_as_light requires a diffuse_light material"
                    ));
                }
            }
        }
        Ok(())
    }
}

impl MaterialConfig {
    fn build(&self) -> Arc<dyn Material> {
        match self {
            Self::Lambertian { texture } => Arc::new(Lambertian::new(texture.build())),
            Self::Metal { albedo, fuzz } => Arc::new(Metal::new(colour(*albedo), *fuzz)),
            Self::Dielectric {
                index_of_refraction,
            } => Arc::new(Dielectric::new(*index_of_refraction)),
            Self::DiffuseLight { texture } => Arc::new(DiffuseLight::new(texture.build())),
        }
    }

    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::Lambertian { texture } | Self::DiffuseLight { texture } => {
                texture.validate(&format!("{field}.texture"))
            }
            Self::Metal { albedo, fuzz } => {
                require_colour(&format!("{field}.albedo"), *albedo)?;
                if !fuzz.is_finite() || !(0. ..=1.).contains(fuzz) {
                    return Err(format!("{field}.fuzz must be between 0 and 1"));
                }
                Ok(())
            }
            Self::Dielectric {
                index_of_refraction,
            } => {
                if !index_of_refraction.is_finite() || *index_of_refraction <= 0. {
                    return Err(format!(
                        "{field}.index_of_refraction must be finite and positive"
                    ));
                }
                Ok(())
            }
        }
    }
}

impl TextureConfig {
    fn build(&self) -> Arc<dyn Texture> {
        match self {
            Self::Solid { color: value } => Arc::new(SolidColour(colour(*value))),
            Self::Checker { even, odd, scale } => Arc::new(CheckerTexture::new_with_colours(
                colour(*even),
                colour(*odd),
                *scale,
            )),
            Self::Noise { scale } => Arc::new(NoiseTexture::new(*scale)),
        }
    }

    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::Solid { color: value } => require_colour(field, *value),
            Self::Checker { even, odd, scale } => {
                require_colour(&format!("{field}.even"), *even)?;
                require_colour(&format!("{field}.odd"), *odd)?;
                if !scale.is_finite() || *scale <= 0. {
                    return Err(format!("{field}.scale must be finite and positive"));
                }
                Ok(())
            }
            Self::Noise { scale } => {
                if !scale.is_finite() {
                    return Err(format!("{field}.scale must be finite"));
                }
                Ok(())
            }
        }
    }
}

impl ShapeConfig {
    fn build(&self, material: Arc<dyn Material>) -> ScenePrimitive {
        match self {
            Self::Sphere { center, radius } => {
                ScenePrimitive::Sphere(Box::new(Sphere::new(point(*center), *radius, material)))
            }
            Self::Plane { point: p, normal } => {
                ScenePrimitive::Plane(Box::new(Plane::new(point(*p), vector(*normal), material)))
            }
            Self::Quad { point: p, u, v } => ScenePrimitive::Quad(Box::new(Quad::new(
                point(*p),
                vector(*u),
                vector(*v),
                material,
            ))),
            Self::Cuboid { min, max } => {
                ScenePrimitive::Cuboid(Box::new(Cuboid::new(point(*min), point(*max), material)))
            }
        }
    }

    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::Sphere { center, radius } => {
                require_finite(&format!("{field}.center"), center)?;
                if !radius.is_finite() || *radius <= 0. {
                    return Err(format!("{field}.radius must be finite and positive"));
                }
            }
            Self::Plane { point: p, normal } => {
                require_finite(&format!("{field}.point"), p)?;
                require_finite(&format!("{field}.normal"), normal)?;
                if vector(*normal).length() <= f64::EPSILON {
                    return Err(format!("{field}.normal must be non-zero"));
                }
                if normal[1] < 0.
                    && normal[0].abs() < f64::EPSILON
                    && normal[2].abs() < f64::EPSILON
                {
                    return Err(format!("{field}.normal cannot point straight down"));
                }
            }
            Self::Quad { point: p, u, v } => {
                require_finite(&format!("{field}.point"), p)?;
                require_finite(&format!("{field}.u"), u)?;
                require_finite(&format!("{field}.v"), v)?;
                let cross = vector(*u).cross(vector(*v));
                if cross.length() <= f64::EPSILON {
                    return Err(format!("{field}.u and {field}.v must span a non-zero area"));
                }
            }
            Self::Cuboid { min, max } => {
                require_finite(&format!("{field}.min"), min)?;
                require_finite(&format!("{field}.max"), max)?;
                if (0..3).any(|axis| (min[axis] - max[axis]).abs() <= f64::EPSILON) {
                    return Err(format!("{field}.min and {field}.max must enclose a volume"));
                }
            }
        }
        Ok(())
    }
}

impl ObjectConfig {
    fn transformation(&self) -> Transformation {
        self.transforms
            .iter()
            .fold(Transformation::default(), |transformation, step| {
                transformation.then(&step.transformation())
            })
    }
}

impl TransformConfig {
    fn transformation(&self) -> Transformation {
        match self {
            Self::Translate { offset } => Translation3::new(offset[0], offset[1], offset[2]).into(),
            Self::Rotate { axis, degrees } => rotation(*degrees, axis.geometry_axis()),
        }
    }

    fn validate(&self, field: &str) -> Result<(), String> {
        match self {
            Self::Translate { offset } => require_finite(&format!("{field}.offset"), offset),
            Self::Rotate { degrees, .. } => {
                if !degrees.is_finite() {
                    return Err(format!("{field}.degrees must be finite"));
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
enum ScenePrimitive {
    Sphere(Box<Sphere>),
    Plane(Box<Plane>),
    Quad(Box<Quad>),
    Cuboid(Box<Cuboid>),
}

impl Hittable for ScenePrimitive {
    fn hit(&self, ray: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        match self {
            Self::Sphere(value) => value.hit(ray, range),
            Self::Plane(value) => value.hit(ray, range),
            Self::Quad(value) => value.hit(ray, range),
            Self::Cuboid(value) => value.hit(ray, range),
        }
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self {
            Self::Sphere(value) => value.pdf_value(origin, direction),
            Self::Plane(value) => value.pdf_value(origin, direction),
            Self::Quad(value) => value.pdf_value(origin, direction),
            Self::Cuboid(value) => value.pdf_value(origin, direction),
        }
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        match self {
            Self::Sphere(value) => value.random(origin, rng),
            Self::Plane(value) => value.random(origin, rng),
            Self::Quad(value) => value.random(origin, rng),
            Self::Cuboid(value) => value.random(origin, rng),
        }
    }
}

impl Bounded for ScenePrimitive {
    fn get_aabbox(&self) -> geometry::aabbox::AABBox {
        match self {
            Self::Sphere(value) => value.get_aabbox(),
            Self::Plane(value) => value.get_aabbox(),
            Self::Quad(value) => value.get_aabbox(),
            Self::Cuboid(value) => value.get_aabbox(),
        }
    }

    fn get_surface_area(&self) -> f64 {
        match self {
            Self::Sphere(value) => value.get_surface_area(),
            Self::Plane(value) => value.get_surface_area(),
            Self::Quad(value) => value.get_surface_area(),
            Self::Cuboid(value) => value.get_surface_area(),
        }
    }
}

impl BoundedHittable for ScenePrimitive {}

const fn point(value: [f64; 3]) -> Point3 {
    Point3::new(value[0], value[1], value[2])
}

const fn vector(value: [f64; 3]) -> Vec3 {
    Vec3::new(value[0], value[1], value[2])
}

const fn colour(value: [f64; 3]) -> Colour {
    Colour::new(value[0], value[1], value[2])
}

fn require_finite(field: &str, values: &[f64]) -> Result<(), String> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(format!("{field} must contain only finite values"))
    }
}

fn require_colour(field: &str, value: [f64; 3]) -> Result<(), String> {
    require_finite(field, &value)?;
    if value.iter().any(|channel| *channel < 0.) {
        return Err(format!("{field} channels must be non-negative"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use geometry::vec3::{Point3, Vec3};
    use rand::{SeedableRng, rngs::SmallRng};
    use shared::ray::Ray;

    use super::SceneConfig;

    const BASIC_SCENE: &str = r#"
[camera]
look_from = [0.0, 1.0, 5.0]
look_at = [0.0, 0.0, 0.0]

[materials.white]
type = "lambertian"
texture = { type = "solid", color = [0.8, 0.8, 0.8] }

[materials.light]
type = "diffuse_light"
texture = { type = "solid", color = [4.0, 4.0, 4.0] }

[[objects]]
material = "white"
[objects.shape]
type = "sphere"
center = [0.0, 0.0, 0.0]
radius = 1.0

[[objects]]
material = "light"
sample_as_light = true
[objects.shape]
type = "quad"
point = [-1.0, 3.0, -1.0]
u = [2.0, 0.0, 0.0]
v = [0.0, 0.0, 2.0]
"#;

    #[test]
    fn parses_and_builds_a_scene_with_named_materials_and_sampled_lights() {
        let scene: SceneConfig = toml::from_str(BASIC_SCENE).unwrap();
        assert!(scene.build_scene().is_ok());
    }

    #[test]
    fn builds_the_documented_cornell_box_example() {
        let scene: SceneConfig =
            toml::from_str(include_str!("../examples/cornell_box.toml")).unwrap();
        assert_eq!(scene.objects.len(), 8);
        let (world, _, _) = scene.build_scene().unwrap();
        let ray = Ray::new(Point3::new(277.5, 277.5, -800.0), Vec3::new(0.0, 0.0, 1.0));
        assert!(world.hit(&ray, 0.001..=f64::INFINITY).is_some());
    }

    #[test]
    fn transformed_sampled_lights_generate_directions_that_hit_the_light() {
        let transformed = BASIC_SCENE.replace(
            "sample_as_light = true",
            "sample_as_light = true\ntransforms = [{ type = \"translate\", offset = [0.5, 0.0, 0.0] }]",
        );
        let scene: SceneConfig = toml::from_str(&transformed).unwrap();
        let (_, lights, _) = scene.build_scene().unwrap();
        let origin = Point3::new(0.0, 1.0, 5.0);
        let mut rng = SmallRng::seed_from_u64(42);
        let direction = lights.random(origin, &mut rng);
        assert!(lights.pdf_value(origin, direction) > 0.0);
    }

    #[test]
    fn rejects_unknown_material_references() {
        let invalid = BASIC_SCENE.replace("material = \"white\"", "material = \"missing\"");
        let scene: SceneConfig = toml::from_str(&invalid).unwrap();
        assert!(
            scene
                .build_scene()
                .unwrap_err()
                .contains("unknown material \"missing\"")
        );
    }

    #[test]
    fn rejects_non_sampleable_light_geometry() {
        let invalid = BASIC_SCENE.replace(
            "type = \"quad\"\npoint = [-1.0, 3.0, -1.0]\nu = [2.0, 0.0, 0.0]\nv = [0.0, 0.0, 2.0]",
            "type = \"plane\"\npoint = [0.0, 3.0, 0.0]\nnormal = [0.0, 1.0, 0.0]",
        );
        let scene: SceneConfig = toml::from_str(&invalid).unwrap();
        assert!(
            scene
                .build_scene()
                .unwrap_err()
                .contains("sample_as_light is supported only")
        );
    }
}
