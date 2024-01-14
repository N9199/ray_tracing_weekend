use std::{
    fs::{read_to_string, File},
    io::{BufWriter, Write},
};

use itertools::iproduct;
use kdam::{par_tqdm, tqdm};
use rand::{
    distributions::{Open01, Uniform},
    rngs::SmallRng,
    thread_rng, SeedableRng,
};
use rayon::prelude::*;

use crate::{
    camera::Camera,
    config::{Config, Image},
    entities::{Plane, Sphere},
    hittable::Hittable,
    hittable_list::HittableList,
    material::{Dialectric, Lambertian, Material, Metal},
    ray::Ray,
    vec3::{Colour, Point3, SampledColour, Vec3},
};

mod camera;
mod config;
mod entities;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod vec3;

fn main() {
    let config = read_to_string("Config.toml").unwrap();
    let mut config = toml::from_str::<Config>(&config).unwrap();
    let Image {
        aspect_ratio,
        image_width,
        image_height,
        samples_per_pixel,
        max_depth,
    } = config.get_image().unwrap();

    // World
    let world = random_scene();

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
    par_tqdm!(process.into_par_iter()).for_each(|(j, i, v)| {
        let i = (i) as f64;
        let j = (j) as f64;
        let image_width = (image_width - 1) as f64;
        let image_height = (image_height - 1) as f64;
        *v = (0..samples_per_pixel)
            .into_par_iter()
            .map(|_| {
                let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                let u = (i + random_f64(&mut rng)) / image_width;
                let v = (j + random_f64(&mut rng)) / image_height;
                let r = cam.get_ray(u, v, &mut rng);
                ray_colour(&r, &world, max_depth as _)
            })
            .reduce(Colour::default, |acc, val| acc + val);
    });

    let mut file = BufWriter::new(File::create("image.ppm").unwrap());
    file.write_fmt(format_args!("P3\n{image_width} {image_height}\n255\n"))
        .unwrap();
    for (j, i) in iproduct!((0..image_height).rev(), 0..image_width) {
        file.write_fmt(format_args!(
            "{}\n",
            SampledColour::from((out[j as usize][i as usize], samples_per_pixel as _))
        ))
        .unwrap();
    }
}

fn ray_colour(r: &Ray, world: &dyn Hittable, depth: u32) -> Colour {
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

fn random_f64<T: rand::Rng>(rng: &mut T) -> f64 {
    rng.sample(Open01)
}

fn random_f64_2<T: rand::Rng>(rng: &mut T) -> f64 {
    let dist = Uniform::new_inclusive(0.5, 1.);
    rng.sample(dist)
}

fn random_scene() -> HittableList {
    let mut world = HittableList::default();
    let ground_material = Box::new(Lambertian::new(Colour::new(0.3, 0.3, 0.3)));
    world.add(Plane::new(
        Point3::new(0., -0.01, 0.),
        Vec3::new(0., 1., 0.),
        ground_material,
    ));

    // world.add(Sphere {
    //     center: Point3::new(0., -10000., 0.),
    //     radius: 10000.,
    //     mat_ptr: ground_material,
    // });

    let material1 = Box::new(Dialectric::new(1.5));
    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
    for a in (-11)..11 {
        for b in (-11)..11 {
            let choose_mat: f64 = random_f64(&mut rng);
            let center = Point3::new(
                (a) as f64 + 0.9 * random_f64(&mut rng),
                0.2,
                (b) as f64 + 0.9 * random_f64(&mut rng),
            );
            if (center - Point3::new(4., 0.2, 0.)).length() > 0.9 {
                let mat: Box<dyn Material> = if choose_mat < 0.8 {
                    let albedo = Colour::new(
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                    ) * Colour::new(
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                        random_f64(&mut rng),
                    );
                    Box::new(Lambertian::new(albedo))
                } else if choose_mat < 0.95 {
                    let albedo = Colour::new(
                        random_f64_2(&mut rng),
                        random_f64_2(&mut rng),
                        random_f64_2(&mut rng),
                    );
                    let fuzz = 1. - random_f64_2(&mut rng);
                    Box::new(Metal::new(albedo, fuzz))
                } else {
                    material1.clone()
                };
                world.add(Sphere {
                    center,
                    radius: 0.2,
                    mat_ptr: mat,
                });
            }
        }
    }
    let material2 = Box::new(Lambertian::new(Colour::new(0.4, 0.2, 0.1)));
    let material3 = Box::new(Metal::new(Colour::new(0.7, 0.6, 0.5), 0.));

    world.add(Sphere {
        center: Point3::new(0., 1., 0.),
        radius: 1.,
        mat_ptr: material1,
    });
    world.add(Sphere {
        center: Point3::new(-4., 1., 0.),
        radius: 1.,
        mat_ptr: material2,
    });
    world.add(Sphere {
        center: Point3::new(4., 1., 0.),
        radius: 1.,
        mat_ptr: material3,
    });

    world
}
