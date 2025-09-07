use std::sync::Arc;

use rand::{Rng as _, SeedableRng, distributions::Standard, rngs::SmallRng, thread_rng};

use crate::{
    camera::CameraBuilder,
    entities::{
        Axis, Cuboid, Plane, Quad, Sphere,
        transformations::{rotation, transform},
    },
    geometry::vec3::{Colour, Point3, Vec3},
    hittable::BoundedHittable,
    hittable_collections::{bvh::BoundedVolumeHierarchy, hittable_list::HittableList},
    material::{Dialectric, DiffuseLight, INVISIBLE_PTR, Lambertian, Material, Metal},
    texture::{CheckerTexture, NoiseTexture},
    utils::random_utils,
};
type Output = (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
);

pub trait SceneGenerator {
    fn generate_scene(&self) -> Output;
}

impl<T> SceneGenerator for T
where
    T: Fn() -> Output,
{
    fn generate_scene(&self) -> Output {
        (self)()
    }
}

pub fn perlin_spheres() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();
    let pertext = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.))));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        pertext.clone(),
    ));

    world.add(Sphere::new(Point3::new(0., 2., 0.), 2., pertext.clone()));

    world.add(Sphere::new(
        Point3::new(-5., 1., 5.),
        1.,
        Arc::new(Lambertian::new_with_colour(Colour::new(1., 0., 0.))), // RED
    ));
    world.add(Sphere::new(
        Point3::new(-5., 1., -5.),
        1.,
        Arc::new(Lambertian::new_with_colour(Colour::new(0., 1., 0.))), // GREEN
    ));
    world.add(Sphere::new(
        Point3::new(5., 1., -5.),
        1.,
        Arc::new(Lambertian::new_with_colour(Colour::new(0., 0., 1.))), // BLUE
    ));
    world.add(Sphere::new(
        Point3::new(5., 1., 5.),
        1.,
        Arc::new(Lambertian::new_with_colour(Colour::new(0.5, 0., 0.5))), // PURPLE
    ));

    let lights = HittableList::default();

    let lookfrom = Point3::new(0.0, 30.0, 0.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist((lookfrom - lookat).length())
        .with_vfov(40.)
        .with_background(Colour::new(1., 1., 1.));

    (Box::new(world), Box::new(lights), cam)
}

pub fn plane() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
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

    let lights = HittableList::default();

    let lookfrom = Point3::new(0.0, 30.0, 0.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist((lookfrom - lookat).length())
        .with_vfov(40.)
        .with_background(Colour::new(1., 1., 1.));

    (Box::new(world), Box::new(lights), cam)
}

pub fn checkered_spheres() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();

    let checker = Arc::new(Lambertian::new(Arc::new(CheckerTexture::new_with_colours(
        Colour::new(0.2, 0.3, 0.1),
        Colour::new(0.9, 0.9, 0.9),
        0.01,
    ))));

    world.add(Sphere::new(Point3::new(0., -10., 0.), 10., checker.clone()));
    world.add(Sphere::new(Point3::new(0., 10., 0.), 10., checker.clone()));

    let mut lights = HittableList::default();

    lights.add(Sphere::new(Point3::new(0., 0., 0.), 0.1, checker.clone()));

    let lookfrom = Point3::new(40.0, 1.0, 0.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist((lookfrom - lookat).length())
        .with_vfov(40.)
        .with_background(Colour::new(1., 1., 1.));

    (Box::new(world), Box::new(lights), cam)
}

pub fn simple() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut lights = HittableList::default();
    let mut world = HittableList::default();
    let invisible_material = INVISIBLE_PTR;
    let ground_material = Arc::new(Lambertian::new_with_colour(Colour::new(0.9, 0.9, 0.9)));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        ground_material,
    ));

    let material1 = Arc::new(Dialectric::new(1.5));
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    const N: isize = 11;
    for a in (-N)..N {
        for b in (-N)..N {
            let choose_mat: f64 = rng.sample::<f64, _>(Standard);
            let center = Point3::new(
                (a) as f64 + 0.9 * rng.sample::<f64, _>(Standard),
                0.2,
                (b) as f64 + 0.9 * rng.sample::<f64, _>(Standard),
            );
            if (center - Point3::new(4., 0.2, 0.)).length() > 0.9 {
                let mat: Arc<dyn Material> = if choose_mat < 0.8 {
                    let albedo = Colour::new(
                        rng.sample::<f64, _>(Standard),
                        rng.sample::<f64, _>(Standard),
                        rng.sample::<f64, _>(Standard),
                    ) * Colour::new(
                        rng.sample::<f64, _>(Standard),
                        rng.sample::<f64, _>(Standard),
                        rng.sample::<f64, _>(Standard),
                    );
                    Arc::new(Lambertian::new_with_colour(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Colour::new(
                        random_utils::random_f64_2(&mut rng),
                        random_utils::random_f64_2(&mut rng),
                        random_utils::random_f64_2(&mut rng),
                    );
                    let fuzz = 1. - random_utils::random_f64_2(&mut rng);
                    Arc::new(Metal::new(albedo, fuzz))
                } else {
                    lights.add(Sphere::new(center, 0.2, invisible_material));
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

    lights.add(Sphere::new(Point3::new(0., 1., 0.), 1., invisible_material));

    let lookfrom = Point3::new(10.0, 5.0, 10.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist((lookfrom - lookat).length())
        .with_vfov(40.)
        .with_background(Colour::new(1., 1., 1.));

    (
        Box::new(BoundedVolumeHierarchy::from(world)),
        Box::new(lights),
        cam,
    )
}

pub fn simple_light() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();
    let pertext = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.))));

    let difflight = Arc::new(DiffuseLight::new_with_colour(Colour::new(4., 4., 4.)));

    // world.add(Sphere::new(
    //     Point3::new(0., -10000., 0.),
    //     10000.,
    //     pertext.clone(),
    // ));
    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        pertext.clone(),
    ));
    // world.add(Quad::new(
    //     Point3::new(-10., 0., -10.),
    //     Vec3::new(20., 0., 0.),
    //     Vec3::new(0., 0., 20.),
    //     pertext.clone(),
    // ));
    world.add(Sphere::new(Point3::new(0., 2., 0.), 2., pertext.clone()));

    world.add(Quad::new(
        Point3::new(3., 1., -2.),
        Vec3::new(2., 0., 0.),
        Vec3::new(0., 2., 0.),
        difflight.clone(),
    ));

    let mut lights = HittableList::default();

    lights.add(Quad::new(
        Point3::new(3., 1., -2.),
        Vec3::new(2., 0., 0.),
        Vec3::new(0., 2., 0.),
        difflight.clone(),
    ));

    // world.add(Sphere::new(Point3::new(0., 7., 0.), 2., difflight.clone()));

    let lookfrom = Point3::new(26.0, 3.0, 6.0);
    let lookat = Point3::new(0.0, 2.0, 0.0);
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist((lookfrom - lookat).length())
        .with_vfov(40.);

    (Box::new(world), Box::new(lights), cam)
}

pub fn cornell_box() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();
    let red = Arc::new(Lambertian::new_with_colour(Colour::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new_with_colour(Colour::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new_with_colour(Colour::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new_with_colour(Colour::new(15., 15., 15.)));

    let _aluminium = Arc::new(Metal::new(Colour::new(0.8, 0.85, 0.88), 0.0));
    let glass = Arc::new(Dialectric::new(1.5));

    world.add(Quad::new(
        Point3::new(555., 0., 0.),
        Vec3::new(0., 555., 0.),
        Vec3::new(0., 0., 555.),
        green.clone(),
    ));
    world.add(Quad::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 555., 0.),
        Vec3::new(0., 0., 555.),
        red.clone(),
    ));
    world.add(Quad::new(
        Point3::new(0., 0., 0.),
        Vec3::new(555., 0., 0.),
        Vec3::new(0., 0., 555.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(0., 555., 0.),
        Vec3::new(555., 0., 0.),
        Vec3::new(0., 0., 555.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(0., 0., 555.),
        Vec3::new(0., 555., 0.),
        Vec3::new(555., 0., 0.),
        white.clone(),
    ));

    world.add(
        transform(
            Cuboid::new(
                Point3::default(),
                Point3::new(165., 330., 165.),
                white.clone(),
            ),
            Vec3::new(265., 0., 295.),
        )
        .transform(rotation(15., Axis::Y)),
    );
    // world.add(
    //     transform(
    //         Cuboid::new(
    //             Point3::default(),
    //             Point3::new(165., 165., 165.),
    //             white.clone(),
    //         ),
    //         Vec3::new(130., 0., 65.),
    //     )
    //     .transform(rotation(-18., Axis::Y)),
    // );
    world.add(Sphere::new(
        Point3::new(190., 90., 190.),
        90.,
        glass.clone(),
    ));

    world.add(Quad::new(
        Point3::new(343., 554., 332.),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    ));

    let mut lights = HittableList::default();

    lights.add(Quad::new(
        Point3::new(343., 554., 332.),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        light.clone(),
    ));
    lights.add(Sphere::new(
        Point3::new(190., 90., 190.),
        90.,
        glass.clone(),
    ));

    let lookfrom = Point3::new(277.5, 277.5, -800.0);
    let lookat = Point3::new(277.5, 277.5, 0.0);
    let aperture = 0.0;
    let cam = CameraBuilder::new()
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_vfov(40.)
        .with_defocus_angle(aperture)
        .with_focus_dist((lookfrom - lookat).length());

    (Box::new(world), Box::new(lights), cam)
}

pub fn debugging_scene() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();

    let pertext = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.))));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        pertext.clone(),
    ));

    world.add(Sphere::new(Point3::new(0., 2., 0.), 2., pertext.clone()));

    let white = Arc::new(Lambertian::new_with_colour(Colour::new(0.75, 0.75, 0.75)));

    world.add(Quad::new(
        Point3::new(6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(-2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., -2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(-6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(-6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., -2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(-6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(-6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., 2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(-2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., 2.),
        white.clone(),
    ));
    world.extend([
        Sphere::new(
            Point3::new(5., 1., 5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0.5, 0., 0.5))), // PURPLE
        ),
        Sphere::new(
            Point3::new(-5., 1., 5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(1., 0., 0.))), // RED
        ),
        Sphere::new(
            Point3::new(-5., 1., -5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0., 1., 0.))), // GREEN
        ),
        Sphere::new(
            Point3::new(5., 1., -5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0., 0., 1.))), // BLUE
        ),
    ]);

    let mut lights = HittableList::default();
    lights.extend([
        Sphere::new(
            Point3::new(5., 1., 5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0.5, 0., 0.5))), // PURPLE
        ),
        Sphere::new(
            Point3::new(-5., 1., 5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(1., 0., 0.))), // RED
        ),
        Sphere::new(
            Point3::new(-5., 1., -5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0., 1., 0.))), // GREEN
        ),
        Sphere::new(
            Point3::new(5., 1., -5.),
            1.,
            Arc::new(DiffuseLight::new_with_colour(Colour::new(0., 0., 1.))), // BLUE
        ),
    ]);

    let lookfrom = Point3::new(0., 20., 0.);
    let lookat = Point3::new(0., 0., 0.);
    let cam = CameraBuilder::new()
        .with_image_width(3)
        .with_image_height(2)
        .with_samples_per_pixel(10)
        .with_max_depth(5)
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist(4.);

    (
        Box::new(BoundedVolumeHierarchy::from(world)),
        Box::new(BoundedVolumeHierarchy::from(lights)),
        cam,
    )
}

pub fn simple_transform() -> (
    Box<dyn BoundedHittable>,
    Box<dyn BoundedHittable>,
    CameraBuilder,
) {
    let mut world = HittableList::default();

    let pertext = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.))));

    world.add(Plane::new(
        Point3::new(0., 0., 0.),
        Vec3::new(0., 1., 0.),
        pertext.clone(),
    ));

    let white = Arc::new(Lambertian::new_with_colour(Colour::new(0.75, 0.75, 0.75)));

    world.add(Quad::new(
        Point3::new(6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(-2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., -2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(-6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(-6., 0., 6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., -2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(-6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(-6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., 2.),
        white.clone(),
    ));

    world.add(Quad::new(
        Point3::new(6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(-2., 0., 0.),
        white.clone(),
    ));
    world.add(Quad::new(
        Point3::new(6., 0., -6.),
        Vec3::new(0., 2., 0.),
        Vec3::new(0., 0., 2.),
        white.clone(),
    ));

    let original = Cuboid::new(
        Point3::default(),
        Point3::new(1., 1., 1.),
        Arc::new(DiffuseLight::new_with_colour(Colour::new(1., 0., 0.))),
    );

    world.add(transform(original.clone(), Vec3::new(-0.5, 0., -0.5)));

    world.add(transform(original.clone(), Vec3::new(2., 0., 2.)));
    world.add(
        transform(original.clone(), Vec3::new(-3., 0., -3.)).transform(rotation(45., Axis::Y)),
    );

    let mut lights = HittableList::default();

    lights.add(transform(original.clone(), Vec3::new(-0.5, 0., -0.5)));

    lights.add(transform(original.clone(), Vec3::new(2., 0., 2.)));
    lights.add(
        transform(original.clone(), Vec3::new(-3., 0., -3.)).transform(rotation(45., Axis::Y)),
    );

    let lookfrom = Point3::new(0., 20., 0.);
    let lookat = Point3::new(0., 0., 0.);
    let cam = CameraBuilder::new()
        .with_image_width(3)
        .with_image_height(2)
        .with_samples_per_pixel(10)
        .with_max_depth(5)
        .with_lookfrom(lookfrom)
        .with_lookat(lookat)
        .with_focus_dist(4.);

    (
        Box::new(BoundedVolumeHierarchy::from(world)),
        Box::new(BoundedVolumeHierarchy::from(lights)),
        cam,
    )
}
