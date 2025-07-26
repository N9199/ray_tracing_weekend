use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use ray_tracing_weekend::{
    entities::Sphere,
    geometry::vec3::{Colour, Point3, Vec3},
    hittable::Hittable,
    material::Lambertian,
    ray::Ray,
};

fn sphere_hits(c: &mut Criterion) {
    let sphere = Sphere::new(
        Point3::default(),
        10.,
        Arc::new(Lambertian::new_with_colour(Colour::new(0., 0., 0.))),
    );

    let mut rng = SmallRng::from_entropy();
    c.bench_function("sphere hit test", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.gen(), rng.gen(), rng.gen());
                Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
            },
            |r| sphere.hit(&r, (0.)..=f64::MAX),
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(hits, sphere_hits);
criterion_main!(hits);
