use criterion::{Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use shared::{
    entities::AABBox,
    geometry::vec3::{Point3, Vec3},
    ray::Ray,
};

fn aabox_hits(c: &mut Criterion) {
    let mut rng = SmallRng::from_entropy();
    let aabox = AABBox::new(
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen(),
        rng.r#gen(),
    );
    let mut group = c.benchmark_group("aabox is_hit");
    group.bench_function("aabox hit test", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.r#gen(), rng.r#gen(), rng.r#gen());
                Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
            },
            |r| aabox.is_hit(&r, (0.)..=f64::MAX),
            criterion::BatchSize::SmallInput,
        )
    });
    group.bench_function("aabox hit test2", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.r#gen(), rng.r#gen(), rng.r#gen());
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
