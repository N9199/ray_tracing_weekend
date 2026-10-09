#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{
    f64::consts::{PI, TAU},
    fmt::Debug,
    ops::{Div, Neg, RangeInclusive},
};

use rand::{Rng, distributions::Standard};

use geometry::{
    aabbox::AABBox,
    bounded::Bounded,
    onb::Onb,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
};

#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat_ptr: DynMaterial,
    aabbox: AABBox,
}

impl Sphere {
    /// # Panics
    /// If `T` fails to be converted into `DynMaterial` this function will panic
    /// If `radius` is not finite or not positive, this function will panic.
    #[must_use]
    pub fn new<T>(center: Point3, radius: f64, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        assert!(
            radius.is_finite() && radius.is_sign_positive(),
            "radius must be finite and positive"
        );
        Sphere {
            center,
            radius,
            mat_ptr: mat_ptr.try_into().unwrap(),
            aabbox: AABBox::new(
                Point3::new(center.x - radius, center.y - radius, center.z - radius),
                Point3::new(center.x + radius, center.y + radius, center.z + radius),
            ),
        }
    }

    #[must_use]
    pub fn get_sphere_uv(point: Point3) -> (f64, f64) {
        (
            libm::atan2(point.z.neg(), point.x).div(TAU),
            point.y.acos().div(PI),
        )
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static SPHERE_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Sphere {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("Sphere");
        let oc = r.get_origin() - self.center;
        let a = r.get_direction().square_length();
        let half_b = r.get_direction().dot(oc);
        let c = oc.square_length() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        (discriminant > 0.).then_some(())?;

        let sqrt_discriminant = discriminant.sqrt();
        let t = {
            let root = (-half_b - sqrt_discriminant) / a;
            if *range.start() <= root && root <= *range.end() {
                root
            } else {
                let root = (-half_b + sqrt_discriminant) / a;
                (*range.start() <= root && root <= *range.end()).then_some(root)?
            }
        };

        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        let (u, v) = Sphere::get_sphere_uv(outward_normal.to_point());
        // dbg!("Sphere hit!", self, r, t);

        #[cfg(feature = "hit_counters")]
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
        match self.hit(&Ray::new(origin, direction), (0.001)..=f64::INFINITY) {
            Some(_) => {
                let distance_squared = (self.center - origin).square_length();
                let cos_theta_max = (1. - (self.radius * self.radius) / distance_squared)
                    .max(0.)
                    .sqrt();
                let solid_angle = 2. * PI * (1. - cos_theta_max);
                1. / solid_angle
            }
            None => 0.,
        }
    }

    // TODO: Look into equivalent but cheaper way to do this
    /// We assume that `origin` is outside the sphere.
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
        self.aabbox
    }

    fn get_surface_area(&self) -> f64 {
        4. * PI * self.radius * self.radius
    }
}

impl BoundedHittable for Sphere {}

#[cfg(test)]
mod tests {
    use geometry::{
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance, assert_vec as assert_vec_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{entities::Sphere, hittable::Hittable, material::INVISIBLE_PTR, ray::Ray};

    const TOLERANCE: f64 = 1e-10;

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_vec(actual: Vec3, expected: Vec3) {
        assert_vec_with_tolerance(actual, expected, TOLERANCE);
    }

    fn assert_point(actual: Point3, expected: Point3) {
        assert_point_with_tolerance(actual, expected, TOLERANCE);
    }

    fn sphere() -> Sphere {
        Sphere::new(Point3::zero(), 1., INVISIBLE_PTR)
    }

    #[test]
    fn sphere_selects_nearest_root_then_far_root_when_range_requires_it() {
        let sphere = sphere();
        let ray = Ray::new(Point3::new(0., 0., -3.), Vec3::new(0., 0., 1.));

        let near = sphere.hit(&ray, 0. ..=f64::INFINITY).unwrap();
        assert_close(near.get_t(), 2.);
        assert_point(near.get_p(), Point3::new(0., 0., -1.));
        assert_vec(near.get_normal(), Vec3::new(0., 0., -1.));
        assert!(near.is_front_face());
        assert_close(near.get_u(), 0.25);
        assert_close(near.get_v(), 0.5);

        let far = sphere.hit(&ray, 4. ..=4.).unwrap();
        assert_close(far.get_t(), 4.);
        assert_point(far.get_p(), Point3::new(0., 0., 1.));
        assert_vec(far.get_normal(), Vec3::new(0., 0., -1.));
        assert!(!far.is_front_face());
        assert!(sphere.hit(&ray, 2.001..=3.999).is_none());
    }

    #[test]
    fn sphere_includes_root_endpoints_and_hits_from_inside() {
        let sphere = sphere();
        let ray = Ray::new(Point3::new(0., 0., -3.), Vec3::new(0., 0., 1.));
        assert_close(sphere.hit(&ray, 2. ..=2.).unwrap().get_t(), 2.);

        let inside = Ray::new(Point3::zero(), Vec3::new(0., 0., 1.));
        let hit = sphere.hit(&inside, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 1.);
        assert_point(hit.get_p(), Point3::new(0., 0., 1.));
        assert_vec(hit.get_normal(), Vec3::new(0., 0., -1.));
        assert!(!hit.is_front_face());
    }

    #[test]
    fn sphere_keeps_t_as_the_unscaled_ray_parameter() {
        let sphere = sphere();
        let ray = Ray::new(Point3::new(0., 0., -3.), Vec3::new(0., 0., 2.));
        let hit = sphere.hit(&ray, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 1.);
        assert_point(hit.get_p(), Point3::new(0., 0., -1.));
    }

    #[test]
    fn sphere_uv_uses_known_outward_unit_normals() {
        let (u, v) = Sphere::get_sphere_uv(Point3::new(1., 0., 0.));
        assert_close(u, 0.);
        assert_close(v, 0.5);

        let (_, north_v) = Sphere::get_sphere_uv(Point3::new(0., 1., 0.));
        assert_close(north_v, 0.);
        let (_, south_v) = Sphere::get_sphere_uv(Point3::new(0., -1., 0.));
        assert_close(south_v, 1.);
    }
}
