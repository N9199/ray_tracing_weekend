use std::{
    f64::consts::{PI, TAU},
    ops::{Div, Neg, RangeInclusive},
    sync::{
        atomic::{self, AtomicU32},
        Arc,
    },
};

use rand::{distributions::Standard, Rng};

use crate::{
    entities::Bounded,
    geometry::{
        onb::Onb,
        vec3::{Point3, Vec3},
    },
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::Material,
    ray::Ray,
    utils::random_utils::UnitSphere,
};

use super::AABBox;

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: Arc<dyn Material>,
    aabox: AABBox,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat_ptr: Arc<dyn Material>) -> Self {
        Sphere {
            center,
            radius,
            mat_ptr,
            aabox: AABBox::new(
                center.get_x() - radius,
                center.get_x() + radius,
                center.get_y() - radius,
                center.get_y() + radius,
                center.get_z() - radius,
                center.get_z() + radius,
            ),
        }
    }

    pub fn get_sphere_uv(point: Point3) -> (f64, f64) {
        (
            point.get_z().neg().atan2(point.get_x()).div(TAU),
            point.get_y().acos().div(PI),
        )
    }
}

#[cfg(debug_assertions)]
pub(crate) static SPHERE_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("Sphere");
        let oc = r.get_origin() - self.center;
        let a = r.get_direction().length_squared();
        let half_b = r.get_direction().dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        (discriminant > 0.).then_some(())?;

        let sqrt_discriminant = discriminant.sqrt();
        let t = {
            let root = (-half_b - sqrt_discriminant) / a;
            if !(*range.start() <= root && root <= *range.end()) {
                let root = (-half_b + sqrt_discriminant) / a;
                (*range.start() <= root && root <= *range.end()).then_some(root)?
            } else {
                root
            }
        };

        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let (u, v) = Sphere::get_sphere_uv(outward_normal);
        // dbg!("Sphere hit!", self, r, t);
        #[cfg(debug_assertions)]
        {
            SPHERE_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
        }
        Some(HitRecord::new(
            r,
            t,
            outward_normal,
            u,
            v,
            self.mat_ptr.as_ref(),
        ))
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self.hit(&Ray::new(origin, direction), (0.)..=f64::INFINITY) {
            Some(rec) => {
                let distance_squared = (self.center - origin).length_squared();
                let cos_theta_max = (1. - self.radius * self.radius / distance_squared).sqrt();
                let solid_angle = 2. * PI * (1. - cos_theta_max);
                1. / solid_angle
            }
            None => 0.,
        }
    }

    // TODO: Look into equivalent but cheaper way to do this
    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        let direction = self.center - origin;
        let distance = direction.length();
        let uvw = Onb::new(direction);

        let r1: f64 = rng.sample(Standard);
        let r2: f64 = rng.sample(Standard);
        let z = 1. + r1 * (f64::sqrt(1. - self.radius * self.radius / (distance * distance)) - 1.);

        let phi = 2. * PI * r2;
        let x = phi.cos() * (1. - z * z).sqrt();
        let y = phi.sin() * (1. - z * z).sqrt();
        uvw.transform(Vec3::new(x, y, z))
    }
}

impl Bounded for Sphere {
    fn get_aabbox(&self) -> AABBox {
        self.aabox
    }

    fn get_surface_area(&self) -> f64 {
        4. * PI * self.radius * self.radius
    }
}

impl BoundedHittable for Sphere {}
