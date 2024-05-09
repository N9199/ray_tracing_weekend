use std::{
    fs::{read_to_string, File},
    io::{BufWriter, Write},
};

use itertools::iproduct;
use kdam::par_tqdm;
use rand::{rngs::SmallRng, thread_rng, SeedableRng};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};
use utils::{checkered_spheres, plane_scene, random_f64, ray_colour, simple_scene};

use crate::{
    camera::Camera,
    config::{Config, Image},
    vec3::{Colour, Point3, SampledColour, Vec3},
};

mod bvh;
mod camera;
mod config;
mod entities;
mod hittable;
mod hittable_list;
mod material;
mod ray;
#[cfg(test)]
mod test;
mod texture;
mod utils;
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
    // let world = simple_scene();
    let world = simple_scene();

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
