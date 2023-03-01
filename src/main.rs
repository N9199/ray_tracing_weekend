use std::{
    fs::File,
    io::{BufWriter, Write},
};

use itertools::iproduct;
use kdam::tqdm;
use rand::{
    distributions::{Open01, Uniform},
    rngs::SmallRng,
    SeedableRng,
};
use rayon::prelude::*;

use crate::{
    camera::Camera,
    hittable::Hittable,
    hittable_list::HittableList,
    material::{Dialectric, Lambertian, Material, Metal},
    ray::Ray,
    sphere::Sphere,
    vec3::{Colour, Point3, SampledColour, Vec3},
};

mod camera;
mod hittable;
mod hittable_list;
mod material;
mod ray;
mod sphere;
mod vec3;

fn main() {
    // Image
    let aspect_ratio = 3. / 2.;
    let image_width = 300 * 4;
    let image_height = 200 * 4; // image_width/aspect_ratio
    let samples_per_pixel = 1000;
    let max_depth = 5;

    // World
    let world = random_scene();

    // Camera
    let lookfrom = Point3::new(13., 2., 3.);
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
    let mut file = BufWriter::new(File::create("image.ppm").unwrap());

    file.write_fmt(format_args!("P3\n{image_width} {image_height}\n255\n"))
        .unwrap();
    let mut out = Vec::with_capacity((image_height * image_width) as usize);
    for (j, i) in tqdm!(iproduct!((0..image_height).rev(), 0..image_width)) {
        let i = f64::from(i);
        let j = f64::from(j);
        let image_width = f64::from(image_width - 1);
        let image_height = f64::from(image_height - 1);
        let pixel_colour = (0..samples_per_pixel)
            .into_par_iter()
            .map(|_| {
                let mut rng = SmallRng::from_entropy();
                let u = (i + random_f64(&mut rng)) / image_width;
                let v = (j + random_f64(&mut rng)) / image_height;
                let r = cam.get_ray(u, v, &mut rng);
                ray_colour(&r, &world, max_depth)
            })
            .reduce(Colour::default, |acc, val| acc + val);
        out.push(pixel_colour);
    }
    for pixel_colour in tqdm!(out.into_iter()) {
        file.write_fmt(format_args!(
            "{}\n",
            SampledColour::from((pixel_colour, samples_per_pixel))
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
    let ground_material = Box::new(Lambertian::new(Colour::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere {
        center: Point3::new(0., -1000., 0.),
        radius: 1000.,
        mat_ptr: ground_material,
    }));

    let material1 = Box::new(Dialectric::new(1.5));
    let mut rng = SmallRng::from_entropy();
    for a in (-11)..11 {
        for b in (-11)..11 {
            let choose_mat: f64 = random_f64(&mut rng);
            let center = Point3::new(
                f64::from(a) + 0.9 * random_f64(&mut rng),
                0.2,
                f64::from(b) + 0.9 * random_f64(&mut rng),
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
                world.add(Box::new(Sphere {
                    center,
                    radius: 0.2,
                    mat_ptr: mat,
                }))
            }
        }
    }
    let material2 = Box::new(Lambertian::new(Colour::new(0.4, 0.2, 0.1)));
    let material3 = Box::new(Metal::new(Colour::new(0.7, 0.6, 0.5), 0.));

    world.add(Box::new(Sphere {
        center: Point3::new(0., 1., 0.),
        radius: 1.,
        mat_ptr: material1,
    }));
    world.add(Box::new(Sphere {
        center: Point3::new(-4., 1., 0.),
        radius: 1.,
        mat_ptr: material2,
    }));
    world.add(Box::new(Sphere {
        center: Point3::new(4., 1., 0.),
        radius: 1.,
        mat_ptr: material3,
    }));

    world
}
