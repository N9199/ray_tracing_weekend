#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU32};
use std::{
    fmt::Debug,
    ops::{Div, RangeInclusive, Sub},
};

use rand::{Rng as _, distributions::Open01};

use geometry::{
    aabbox::AABBox,
    bounded::Bounded,
    vec3::{Point3, Vec3},
};

use crate::{
    hittable::{BoundedHittable, HitRecord, Hittable},
    material::DynMaterial,
    ray::Ray,
    utils::random_utils::UNIT,
};

#[derive(Debug, Clone)]
pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    normal: Vec3,
    mat_ptr: DynMaterial,
    aabbox: AABBox,
    area: f64,
}

impl Quad {
    /// # Panics
    /// If `T` fails to be converted into `DynMaterial` this function will panic
    #[must_use]
    pub fn new<T>(q: Point3, u: Vec3, v: Vec3, mat_ptr: T) -> Self
    where
        T: TryInto<DynMaterial>,
        <T as TryInto<DynMaterial>>::Error: Debug,
    {
        let mat_ptr = mat_ptr.try_into().unwrap();
        let aabbox = AABBox::from_points([(q + (u + v) * 0.5), q, q + v, q + u, q + u + v]);
        let normal = u.cross(v);
        let w = normal.div(normal.square_length());
        let area = normal.length();
        let normal = normal / area;
        Self {
            q,
            u,
            v,
            w,
            normal,
            mat_ptr,
            aabbox,
            area,
        }
    }

    fn get_quad_uv(&self, point: Point3) -> (f64, f64) {
        (
            point.sub(self.q).cross(self.v).dot(self.w),
            self.u.cross(point.sub(self.q)).dot(self.w),
        )
    }
}

impl Bounded for Quad {
    fn get_aabbox(&self) -> AABBox {
        self.aabbox
    }

    fn get_surface_area(&self) -> f64 {
        self.area
    }
}

#[cfg(feature = "hit_counters")]
pub(crate) static QUAD_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl Hittable for Quad {
    fn hit(&self, r: &Ray, range: RangeInclusive<f64>) -> Option<HitRecord<'_>> {
        // dbg!("Quad");
        let denom = r.get_direction().dot(self.normal);
        (denom.abs() > f64::EPSILON).then_some(())?;
        let t = -(r.get_origin() - self.q).dot(self.normal).div(denom);
        (range.contains(&t)).then_some(())?;
        let point = r.at(t);
        let (u, v) = self.get_quad_uv(point);
        // dbg!(UNIT.contains(&u), UNIT.contains(&v), u, v, point);
        (UNIT.contains(&u) && UNIT.contains(&v)).then(|| {
            // dbg!("Quad Hit!");

            #[cfg(feature = "hit_counters")]
            QUAD_HIT_COUNTER.fetch_add(1, atomic::Ordering::Relaxed);
            // dbg!(self.mat_ptr.as_ref());
            HitRecord::new(r, t, self.normal, u, v, self.mat_ptr.as_ref())
        })
    }

    fn pdf_value(&self, origin: Point3, direction: Vec3) -> f64 {
        match self.hit(&Ray::new(origin, direction), (0.001)..=f64::INFINITY) {
            Some(record) => {
                let distance_squared = record.get_t() * record.get_t() * direction.square_length();
                let cosine = direction
                    .dot(record.get_normal())
                    .div(direction.length())
                    .abs();
                distance_squared / (cosine * self.area)
            }
            None => 0.,
        }
    }

    fn random(&self, origin: Point3, rng: &mut dyn rand::RngCore) -> Vec3 {
        let p =
            self.q + self.u * rng.sample::<f64, _>(Open01) + self.v * rng.sample::<f64, _>(Open01);
        p - origin
    }
}

impl BoundedHittable for Quad {
    // fn is_aabbox_hit(&self, _: &Ray, _: RangeInclusive<f64>) -> bool {

    //     true
    // }
}

#[cfg(test)]
mod tests {
    use geometry::{
        test_utils::{
            assert_close as assert_close_with_tolerance,
            assert_point as assert_point_with_tolerance, assert_vec as assert_vec_with_tolerance,
        },
        vec3::{Point3, Vec3},
    };

    use crate::{entities::Quad, hittable::Hittable, material::INVISIBLE_PTR, ray::Ray};

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

    fn quad() -> Quad {
        Quad::new(
            Point3::zero(),
            Vec3::new(2., 0., 0.),
            Vec3::new(0., 2., 0.),
            INVISIBLE_PTR,
        )
    }

    #[test]
    fn quad_is_two_sided_and_orients_hit_record_against_ray() {
        let quad = quad();
        let forward = Ray::new(Point3::new(1., 1., -2.), Vec3::new(0., 0., 1.));
        let hit = quad.hit(&forward, 0. ..=f64::INFINITY).unwrap();
        assert_close(hit.get_t(), 2.);
        assert_point(hit.get_p(), Point3::new(1., 1., 0.));
        assert_close(hit.get_u(), 0.5);
        assert_close(hit.get_v(), 0.5);
        assert!(!hit.is_front_face());
        assert_vec(hit.get_normal(), Vec3::new(0., 0., -1.));

        let reverse = Ray::new(Point3::new(1., 1., 2.), Vec3::new(0., 0., -1.));
        let hit = quad.hit(&reverse, 0. ..=f64::INFINITY).unwrap();
        assert!(hit.is_front_face());
        assert_vec(hit.get_normal(), Vec3::new(0., 0., 1.));
    }

    #[test]
    fn quad_includes_edges_and_corners_but_rejects_points_outside() {
        let quad = quad();
        for (x, y, expected_u, expected_v) in [
            (0., 1., 0., 0.5),
            (2., 1., 1., 0.5),
            (1., 0., 0.5, 0.),
            (1., 2., 0.5, 1.),
            (0., 0., 0., 0.),
            (2., 2., 1., 1.),
        ] {
            let ray = Ray::new(Point3::new(x, y, -1.), Vec3::new(0., 0., 1.));
            let hit = quad.hit(&ray, 0. ..=1.).unwrap();
            assert_close(hit.get_u(), expected_u);
            assert_close(hit.get_v(), expected_v);
        }

        let outside = Ray::new(Point3::new(2.000_001, 1., -1.), Vec3::new(0., 0., 1.));
        assert!(quad.hit(&outside, 0. ..=2.).is_none());
    }

    #[test]
    fn quad_respects_closed_range_and_rejects_parallel_rays() {
        let quad = quad();
        let ray = Ray::new(Point3::new(1., 1., -2.), Vec3::new(0., 0., 1.));
        assert_close(quad.hit(&ray, 2. ..=2.).unwrap().get_t(), 2.);
        assert!(quad.hit(&ray, 0. ..=1.999).is_none());

        let parallel = Ray::new(Point3::new(1., 1., 1.), Vec3::new(1., 0., 0.));
        assert!(quad.hit(&parallel, 0. ..=f64::INFINITY).is_none());
        let threshold = Ray::new(Point3::new(1., 1., 1.), Vec3::new(0., 0., f64::EPSILON));
        assert!(quad.hit(&threshold, 0. ..=f64::INFINITY).is_none());
    }
}
