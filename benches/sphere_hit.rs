use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use ray_tracing_weekend::{
    hittable::Hittable,
    material::Lambertian,
    ray::Ray,
    entities::Sphere,
    vec3::{Colour, Point3, Vec3},
};

fn sphere_hits(c: &mut Criterion) {
    let sphere = Sphere {
        center: Point3::default(),
        radius: 10.,
        mat_ptr: Box::new(Lambertian::new(Colour::new(0., 0., 0.))),
    };

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
