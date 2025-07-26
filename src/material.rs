use core::f64;
#[cfg(debug_assertions)]
use std::sync::atomic::AtomicU32;
use std::{
    f64::consts::PI,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use rand::{distributions::Open01, rngs::SmallRng, thread_rng, Rng, SeedableRng};

use crate::{
    geometry::vec3::{Colour, Point3},
    hittable::HitRecord,
    pdf::{CosinePdf, Pdf, SpherePdf},
    ray::Ray,
    texture::{self, SolidColour, Texture},
    utils::random_utils::UnitSphere,
};

#[derive(Debug)]
pub enum ScatterReflect {
    Reflect(Ray),
    Scatter(Arc<dyn Pdf>),
}

#[derive(Debug)]
pub struct ScatterRecord {
    pub attenuation: Colour,
    pub scatter_reflect: ScatterReflect,
}

pub trait Material: Sync + Send + Debug {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _point: Point3) -> Colour {
        Colour::new(0., 0., 0.)
    }

    fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        0.
    }
}

pub struct Lambertian {
    texture: Arc<dyn Texture>,
    rng: Mutex<SmallRng>,
}

impl Debug for Lambertian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lambertian")
            .field("texture", &self.texture)
            .finish()
    }
}

impl Clone for Lambertian {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Lambertian {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self {
            texture,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }

    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord<'_>) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self
                .texture
                .get_colour(rec.get_u(), rec.get_v(), rec.get_p()),
            scatter_reflect: ScatterReflect::Scatter(Arc::new(CosinePdf::new(rec.get_normal()))),
        })
    }

    fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        let cos_theta = rec.get_normal().dot(scattered.get_direction().unit_vec()) / PI;
        cos_theta.max(0.)
    }
}

pub struct Metal {
    albedo: Colour,
    fuzz: f64,
    rng: Mutex<SmallRng>,
}

impl Debug for Metal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metal")
            .field("albedo", &self.albedo)
            .field("fuzz", &self.fuzz)
            .finish()
    }
}

impl Clone for Metal {
    fn clone(&self) -> Self {
        Self {
            albedo: self.albedo,
            fuzz: self.fuzz,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Metal {
    pub fn new(albedo: Colour, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<ScatterRecord> {
        let reflected = ray_in.get_direction().unit_vec().reflect(rec.get_normal());
        let reflected = Ray::new(
            rec.get_p(),
            reflected + self.rng.lock().unwrap().sample(UnitSphere) * self.fuzz,
        );
        (reflected.get_direction().dot(rec.get_normal()) > 0.).then_some(ScatterRecord {
            attenuation: self.albedo,
            scatter_reflect: ScatterReflect::Reflect(reflected),
        })
    }
}

pub struct Dialectric {
    index_of_refraction: f64,
    rng: Mutex<SmallRng>,
}

impl Debug for Dialectric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialectric")
            .field("index_of_refraction", &self.index_of_refraction)
            .finish()
    }
}

impl Clone for Dialectric {
    fn clone(&self) -> Self {
        Self {
            index_of_refraction: self.index_of_refraction,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Dialectric {
    pub fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
    #[inline]
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0 = r0 * r0;
        r0 + (1. - r0) * ((1. - cosine).powi(5))
    }
}

impl Material for Dialectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<ScatterRecord> {
        let refraction_ratio = if rec.is_front_face() {
            self.index_of_refraction.recip()
        } else {
            self.index_of_refraction
        };
        let unit_direction = ray_in.get_direction().unit_vec();

        let cos_theta = unit_direction.dot(-rec.get_normal()).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.;
        let direction = if cannot_refract
            || Self::reflectance(cos_theta, refraction_ratio)
                > (*self.rng.lock().unwrap()).sample(Open01)
        {
            unit_direction.reflect(rec.get_normal())
        } else {
            unit_direction.refract(rec.get_normal(), refraction_ratio)
        };

        Some(ScatterRecord {
            attenuation: Colour::new(1., 1., 1.),
            scatter_reflect: ScatterReflect::Reflect(Ray::new(rec.get_p(), direction)),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}
#[cfg(debug_assertions)]
pub(crate) static LIGHT_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl DiffuseLight {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, point: Point3) -> Colour {
        #[cfg(debug_assertions)]
        LIGHT_HIT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.texture.get_colour(u, v, point)
    }
}

#[derive(Debug, Clone)]
pub struct Isotropic {
    texture: Arc<dyn Texture>,
}

impl Isotropic {
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self
                .texture
                .get_colour(rec.get_u(), rec.get_v(), rec.get_p()),
            scatter_reflect: ScatterReflect::Scatter(Arc::new(SpherePdf)),
        })
    }

    fn emitted(&self, _u: f64, _v: f64, _point: Point3) -> Colour {
        Colour::new(0., 0., 0.)
    }

    fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        1. / (4. * PI)
    }
}
