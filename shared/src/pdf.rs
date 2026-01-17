use std::{f64::consts::PI, fmt::Debug};

use rand::{Rng, distributions::Standard};

use geometry::{
    onb::Onb,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::Hittable,
    utils::random_utils::{CosineWeightedHemisphere, UnitSphere},
};

pub trait Pdf: Send + Sync + Debug {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3;
}

#[derive(Debug)]
pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _direction: &Vec3) -> f64 {
        1. / (4. * PI)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        rng.sample(UnitSphere)
    }
}

#[derive(Debug)]
pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: Vec3) -> Self {
        Self { uvw: Onb::new(w) }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.normalize().dot(self.uvw.get_w()) / PI;
        cosine_theta.max(0.)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        self.uvw.transform(rng.sample(CosineWeightedHemisphere))
    }
}

#[derive(Debug)]
pub struct HittablePdf<'a> {
    objects: &'a dyn Hittable,
    origin: Point3,
}

impl<'a> HittablePdf<'a> {
    pub const fn new(objects: &'a dyn Hittable, origin: Point3) -> Self {
        Self { objects, origin }
    }
}

impl<'a> Pdf for HittablePdf<'a> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(self.origin, *direction)
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        self.objects.random(self.origin, rng)
    }
}

#[derive(Debug)]
pub struct MixturePdf<'a, 'b> {
    pdf1: &'a dyn Pdf,
    pdf2: &'b dyn Pdf,
}

impl<'a, 'b> MixturePdf<'a, 'b> {
    pub const fn new(pdf1: &'a dyn Pdf, pdf2: &'b dyn Pdf) -> Self {
        Self { pdf1, pdf2 }
    }
}

impl<'a, 'b> Pdf for MixturePdf<'a, 'b> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.pdf1.value(direction) * 0.5 + self.pdf2.value(direction) * 0.5
    }

    fn generate(&self, rng: &mut dyn rand::RngCore) -> Vec3 {
        if rng.sample::<f64, _>(Standard) < 0.5 {
            self.pdf1.generate(rng)
        } else {
            self.pdf2.generate(rng)
        }
    }
}
