use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use ray_tracing_weekend::{
    entities::AABBox,
    geometry::vec3::{Point3, Vec3},
    ray::Ray,
};

fn aabox_hits(c: &mut Criterion) {
    let mut rng = SmallRng::from_entropy();
    let aabox = AABBox::new(
        rng.gen(),
        rng.gen(),
        rng.gen(),
        rng.gen(),
        rng.gen(),
        rng.gen(),
    );
    let mut group = c.benchmark_group("aabox is_hit");
    group.bench_function("aabox hit test", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.gen(), rng.gen(), rng.gen());
                Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
            },
            |r| aabox.is_hit(&r, (0.)..=f64::MAX),
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function("aabox hit test2", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.gen(), rng.gen(), rng.gen());
                Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
            },
            |r| aabox.is_hit2(&r, (0.)..=f64::MAX),
            criterion::BatchSize::SmallInput,
        )
    });
    group.finish();
}

criterion_group!(hits, aabox_hits);
criterion_main!(hits);
