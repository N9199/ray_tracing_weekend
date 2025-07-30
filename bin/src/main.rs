use std::{
    fs::{File, read_to_string},
    io::{BufWriter, Write as _},
};

use crate::{
    cli::{Args, Scenes},
    config::{Config, Image},
};

use clap::Parser;

use shared::scenes::{
    SceneGenerator, checkered_spheres, cornell_box, debugging_scene, perlin_spheres, plane, simple,
    simple_light, simple_transform,
};

mod config;
mod cli {
    use clap::{Parser, ValueEnum};
    #[derive(Debug, Parser)]
    pub struct Args {
        pub scene: Scenes,
        #[arg(long)]
        pub debug: bool,
    }

    #[derive(Debug, ValueEnum, Clone, Copy)]
    pub enum Scenes {
        CornellBox,
        Debug,
        CheckeredSpheres,
        PerlinSpheres,
        Plane,
        Simple,
        SimpleLight,
        SimpleTransform,
    }
}

fn get_scene_generator(scene: Scenes) -> &'static dyn SceneGenerator {
    match scene {
        cli::Scenes::CornellBox => &cornell_box,
        cli::Scenes::Debug => &debugging_scene,
        cli::Scenes::CheckeredSpheres => &checkered_spheres,
        cli::Scenes::PerlinSpheres => &perlin_spheres,
        cli::Scenes::Plane => &plane,
        cli::Scenes::Simple => &simple,
        cli::Scenes::SimpleLight => &simple_light,
        cli::Scenes::SimpleTransform => &simple_transform,
    }
}

fn main() {
    let args = Args::parse();

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
    let scene_generator = get_scene_generator(args.scene);
    let (world, lights, cam) = scene_generator.generate_scene();

    // Camera
    let cam = cam
        .with_vfov(40.)
        .with_aspect_ratio(aspect_ratio)
        .with_max_depth(max_depth as _)
        .with_image_width(image_width)
        .with_image_height(image_height)
        .with_samples_per_pixel(samples_per_pixel)
        .build();

    // Render
    let out = if args.debug {
        cam.render_debug(world.as_ref(), lights.as_ref())
    } else {
        cam.render(world.as_ref(), lights.as_ref())
    };

    // Temp
    let file_name = "image.ppm";
    let mut file = BufWriter::new(File::create(file_name).unwrap());
    file.write_fmt(format_args!("P3\n{image_width} {image_height}\n255\n"))
        .unwrap();
    for (j, i, val) in out
        .into_iter()
        .enumerate()
        .rev()
        .flat_map(|(j, v)| v.into_iter().enumerate().map(move |(i, val)| (j, i, val)))
    {
        #[cfg(debug_assertions)]
        dbg!(j, i);

        file.write_fmt(format_args!("{val}\n")).unwrap();
    }
}
