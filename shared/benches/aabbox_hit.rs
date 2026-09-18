use criterion::{criterion_group, criterion_main, Criterion};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use geometry::{
    aabbox::AABBox,
    vec3::{Point3, Vec3},
};

use shared::{hittable::AABBoxHit as _, ray::Ray};

fn aabbox_hits(c: &mut Criterion) {
    let mut rng = SmallRng::from_entropy();
    let aabbox = AABBox::new(
        Point3::new(rng.r#gen(), rng.r#gen(), rng.r#gen()),
        Point3::new(rng.r#gen(), rng.r#gen(), rng.r#gen()),
    );
    let mut group = c.benchmark_group("aabbox is_hit");
    group.bench_function("aabbox hit test", |b| {
        b.iter_batched(
            || {
                let (x, y, z) = (rng.r#gen(), rng.r#gen(), rng.r#gen());
                Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
            },
            |r| aabbox.is_hit(&r, (0.)..=f64::MAX),
            criterion::BatchSize::SmallInput,
        );
    });
    // group.bench_function("aabbox hit test2", |b| {
    //     b.iter_batched(
    //         || {
    //             let (x, y, z) = (rng.r#gen(), rng.r#gen(), rng.r#gen());
    //             Ray::new(Point3::new(x, y, z), Vec3::new(-x, -y, -z))
    //         },
    //         |r| aabbox.is_hit2(&r, (0.)..=f64::MAX),
    //         criterion::BatchSize::SmallInput,
    //     )
    // });
    group.finish();
}

criterion_group!(hits, aabbox_hits);
criterion_main!(hits);
