#![warn(clippy::missing_const_for_fn)]
#![allow(unused, dead_code)]
use std::{fs::read_to_string, path::Path};

use clap::Parser;
use cli::Args;
use config::{Config, Image};
use scenes::{
    checkered_spheres, cornell_box, debugging_scene, perlin_spheres, plane, simple, simple_light,
    simple_transform,
};

use crate::{cli::Scenes, scenes::SceneGenerator};

mod bvh;
mod camera;
mod config;
mod entities;
mod geometry;
mod hittable;
mod hittable_list;
mod material;
mod pdf;
mod perlin;
mod ray;
mod texture;
mod utils;

mod scenes;
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
    if args.debug {
        cam.render_debug::<&Path>(world.as_ref(), lights.as_ref(), None);
    } else {
        cam.render::<&Path>(world.as_ref(), lights.as_ref(), None);
    }
}
