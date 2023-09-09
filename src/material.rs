use std::sync::Mutex;

use rand::{distributions::Open01, rngs::SmallRng, Rng, SeedableRng, thread_rng};

use crate::{
    hittable::HitRecord,
    ray::Ray,
    vec3::{get_in_unit_sphere, get_unit_vec, Colour},
};

pub trait Material: Sync + Send {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<(Ray, Colour)>;
}

#[derive(Debug)]
pub struct Lambertian {
    albedo: Colour,
    rng: Mutex<SmallRng>,
}

impl Clone for Lambertian {
    fn clone(&self) -> Self {
        Self {
            albedo: self.albedo,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Lambertian {
    pub fn new(albedo: Colour) -> Self {
        Self {
            albedo,
            rng: Mutex::new(SmallRng::from_rng(thread_rng()).unwrap()),
        }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord<'_>) -> Option<(Ray, Colour)> {
        let mut rng = self.rng.lock().unwrap();
        let mut scatter_direction = rec.get_normal() + get_unit_vec(&mut *rng);
        if scatter_direction.is_near_zero() {
            scatter_direction = rec.get_normal();
        }
        Some((Ray::new(rec.get_p(), scatter_direction), self.albedo))
    }
}

#[derive(Debug)]
pub struct Metal {
    albedo: Colour,
    fuzz: f64,
    rng: Mutex<SmallRng>,
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
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<(Ray, Colour)> {
        let reflected = ray_in.get_direction().unit_vec().reflect(rec.get_normal());
        let scattered = Ray::new(
            rec.get_p(),
            reflected + get_in_unit_sphere(&mut *self.rng.lock().unwrap()) * self.fuzz,
        );
        (scattered.get_direction().dot(rec.get_normal()) > 0.).then_some((scattered, self.albedo))
    }
}

#[derive(Debug)]
pub struct Dialectric {
    index_of_refraction: f64,
    rng: Mutex<SmallRng>,
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
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord<'_>) -> Option<(Ray, Colour)> {
        let refraction_ratio = if rec.is_front_face() {
            1. / self.index_of_refraction
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

        Some((Ray::new(rec.get_p(), direction), Colour::new(1., 1., 1.)))
    }
}
