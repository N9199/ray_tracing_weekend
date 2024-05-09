use kdam::tqdm;
use rand::{rngs::SmallRng, thread_rng, SeedableRng as _};

use crate::{
    camera::Camera,
    utils::{plane_scene, random_f64, ray_colour, simple_scene},
    vec3::{Colour, Point3, Vec3},
};

#[test]
fn plane_test(){
    let image_width = 3;
    let image_height = 2;
    let aspect_ratio = image_height as f64 / image_width as f64;
    let samples_per_pixel = 10;
    let max_depth = 3;

    // World
    let world = plane_scene();
    dbg!(&world);
    // Camera
    let lookfrom = Point3::new(-13., 2., 3.);
    let lookat = Point3::new(0., 0., 0.);
    let vup = Vec3::new(0., 1., 0.);
    let dist_to_focus = 10.;
    let aperture = 0.1;
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    // Render
    let mut out: Vec<Vec<_>> = (0..image_height)
        .map(|_| (0..image_width).map(|_| Colour::default()).collect())
        .collect();
    let process: Vec<_> = out
        .iter_mut()
        .enumerate()
        .flat_map(|(j, vec)| {
            vec.iter_mut()
                .enumerate()
                .map(move |(i, v)| (j as u32, i as u32, v))
        })
        .collect();
    tqdm!(process.into_iter()).for_each(|(j, i, v)| {
        let i = (i) as f64;
        let j = (j) as f64;
        let image_width = (image_width - 1) as f64;
        let image_height = (image_height - 1) as f64;
        *v = (0..samples_per_pixel)
            // .into_par_iter()
            .map(|_| {
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let u = (i + random_f64(&mut rng)) / image_width;
                let v = (j + random_f64(&mut rng)) / image_height;
                let r = cam.get_ray(u, v, &mut rng);
                ray_colour(&r, &world, max_depth as _)
            })
            .fold(Colour::default(), |acc, val| acc + val);
    });
}

#[test]
fn small_test() {
    let image_width = 3;
    let image_height = 2;
    let aspect_ratio = image_height as f64 / image_width as f64;
    let samples_per_pixel = 10;
    let max_depth = 3;

    // World
    let world = simple_scene();
    dbg!(&world);
    dbg!(world.depth());
    dbg!(world.node_count());
    // Camera
    let lookfrom = Point3::new(-13., 2., 3.);
    let lookat = Point3::new(0., 0., 0.);
    let vup = Vec3::new(0., 1., 0.);
    let dist_to_focus = 10.;
    let aperture = 0.1;
    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.,
        aspect_ratio,
        aperture,
        dist_to_focus,
    );

    // Render
    let mut out: Vec<Vec<_>> = (0..image_height)
        .map(|_| (0..image_width).map(|_| Colour::default()).collect())
        .collect();
    let process: Vec<_> = out
        .iter_mut()
        .enumerate()
        .flat_map(|(j, vec)| {
            vec.iter_mut()
                .enumerate()
                .map(move |(i, v)| (j as u32, i as u32, v))
        })
        .collect();
    tqdm!(process.into_iter()).for_each(|(j, i, v)| {
        let i = (i) as f64;
        let j = (j) as f64;
        let image_width = (image_width - 1) as f64;
        let image_height = (image_height - 1) as f64;
        *v = (0..samples_per_pixel)
            // .into_par_iter()
            .map(|_| {
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let u = (i + random_f64(&mut rng)) / image_width;
                let v = (j + random_f64(&mut rng)) / image_height;
                let r = cam.get_ray(u, v, &mut rng);
                ray_colour(&r, &world, max_depth as _)
            })
            .fold(Colour::default(), |acc, val| acc + val);
    });
}
