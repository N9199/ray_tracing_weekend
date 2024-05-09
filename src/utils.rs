use std::sync::Arc;

use rand::{
    distributions::{Open01, Uniform},
    rngs::SmallRng,
    thread_rng, SeedableRng,
};

use crate::{
    bvh::BoundedVolumeHierarchy,
    entities::{Plane, Sphere},
    hittable::{BoundedHittable, Hittable},
    hittable_list::HittableList,
    material::{Dialectric, Lambertian, Material, Metal},
    ray::Ray,
    texture::CheckerTexture,
    vec3::{Colour, Point3, Vec3},
};

pub mod slice {
    use std::fmt::Debug;

    use crate::{
        entities::{AABBox, Bounded},
        hittable::{BoundedHittable, Hittable},
    };

    #[repr(C)]
    pub struct Slice<T> {
        ptr: *mut T,
        len: usize,
    }

    // impl<T> Slice<T> {
    //     pub fn from_raw_parts(ptr: *mut T, len: usize) -> Self {
    //         Self { ptr, len }
    //     }
    // }
    impl<T> Slice<T>
    where
        T: Bounded + 'static,
    {
        pub fn get_aabboxes(&self) -> impl Iterator<Item = AABBox> {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .map(|v| v.get_aabbox())
        }
    }

    impl<T> Debug for Slice<T>
    where
        T: Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }.fmt(f)
        }
    }

    unsafe impl<T> Sync for Slice<T> {}
    unsafe impl<T> Send for Slice<T> {}

    impl<T> Bounded for Slice<T>
    where
        T: Bounded,
    {
        fn get_aabbox(&self) -> AABBox {
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
                .iter()
                .map(|obj| obj.get_aabbox())
                .reduce(|mut acc, e| {
                    acc.enclose(&e);
                    acc
                })
                .expect("Slice shouldn't be empty")
        }

        fn get_surface_area(&self) -> f64 {
            unsafe { std::slice::from_raw_parts(self.ptr.cast_const(), self.len) }
                .iter()
                .map(T::get_surface_area)
                .sum()
        }
    }

    impl<T> Hittable for Slice<T>
    where
        T: BoundedHittable,
    {
        fn hit(
            &self,
            r: &crate::ray::Ray,
            range: std::ops::RangeInclusive<f64>,
        ) -> Option<crate::hittable::HitRecord<'_>> {
            let &start = range.start();
            let &end = range.end();
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
                .iter()
                .filter_map(|obj| {
                    (obj.is_aabbox_hit(r, start..=end))
                        .then(|| obj.hit(r, start..=end))
                        .flatten()
                })
                .min_by(|a, b| a.get_t().total_cmp(&b.get_t()))
        }
    }
}

pub fn ray_colour(r: &Ray, world: &dyn Hittable, depth: u32) -> Colour {
    if depth == 0 {
        return Colour::default();
    }
    if let Some(rec) = world.hit(r, (f64::EPSILON)..=f64::INFINITY) {
        if let Some((scattered, attenuation)) = rec.get_material().scatter(r, &rec) {
            return attenuation * ray_colour(&scattered, world, depth - 1);
        }
        return Colour::default();
    }
    let unit_direction = r.get_direction().unit_vec();
    let t = 0.5 * (unit_direction.get_y() + 1.);
    (Vec3::new(1., 1., 1.) * (1. - t) + Vec3::new(0.5, 0.7, 1.) * t).into()
}

pub fn random_f64<T: rand::Rng>(rng: &mut T) -> f64 {
    rng.sample(Open01)
}

fn random_f64_2<T: rand::Rng>(rng: &mut T) -> f64 {
    let dist = Uniform::new_inclusive(0.5, 1.);
    rng.sample(dist)
}

pub fn plane_scene() -> impl BoundedHittable {
    let mut world = HittableList::default();
    let checker = Arc::new(Lambertian::new(Arc::new(CheckerTexture::new_with_colours(
        Colour::new(0.2, 0.3, 0.1),
        Colour::new(0.9, 0.9, 0.9),
        0.32,
    ))));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        checker,
    ));

    world
}

pub fn checkered_spheres() -> impl BoundedHittable {
    let mut world = HittableList::default();

    let checker = Arc::new(Lambertian::new(Arc::new(CheckerTexture::new_with_colours(
        Colour::new(0.2, 0.3, 0.1),
        Colour::new(0.9, 0.9, 0.9),
        0.01,
    ))));

    world.add(Sphere::new(Point3::new(0., -10., 0.), 10., checker.clone()));
    world.add(Sphere::new(Point3::new(0., 10., 0.), 10., checker.clone()));

    world
}

pub fn simple_scene() -> BoundedVolumeHierarchy {
    let mut world = HittableList::default();
    let ground_material = Arc::new(Lambertian::new_with_colour(Colour::new(0.9, 0.9, 0.9)));
    let checker = Arc::new(Lambertian::new(Arc::new(CheckerTexture::new_with_colours(
        Colour::new(0.2, 0.3, 0.1),
        Colour::new(0.9, 0.9, 0.9),
        0.32,
    ))));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        checker,
    ));

    let material1 = Arc::new(Dialectric::new(1.5));
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    const N: isize = 11;
    for a in (-N)..N {
        for b in (-N)..N {
            let choose_mat: f64 = random_f64(&mut rng);
            let center = Point3::new(
                (a) as f64 + 0.9 * random_f64(&mut rng),
                0.2,
                (b) as f64 + 0.9 * random_f64(&mut rng),
            );
            if (center - Point3::new(4., 0.2, 0.)).length() > 0.9 {
                let mat: Arc<dyn Material> = if choose_mat < 0.8 {
                    let albedo = Colour::new(
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                    ) * Colour::new(
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                    );
                    Arc::new(Lambertian::new_with_colour(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Colour::new(
                        random_f64_2(&mut rng),
                        random_f64_2(&mut rng),
                        random_f64_2(&mut rng),
                    );
                    let fuzz = 1. - random_f64_2(&mut rng);
                    Arc::new(Metal::new(albedo, fuzz))
                } else {
                    material1.clone()
                };
                world.add(Sphere::new(center, 0.2, mat));
            }
        }
    }
    let material2 = Arc::new(Lambertian::new_with_colour(Colour::new(0.4, 0.2, 0.1)));
    let material3 = Arc::new(Metal::new(Colour::new(0.7, 0.6, 0.5), 0.));

    world.add(Sphere::new(Point3::new(0., 1., 0.), 1., material1));
    world.add(Sphere::new(Point3::new(-4., 1., 0.), 1., material2));
    world.add(Sphere::new(Point3::new(4., 1., 0.), 1., material3));

    BoundedVolumeHierarchy::from(world)
}
