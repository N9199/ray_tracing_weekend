use std::{
    error::Error,
    fs::{read_to_string, File},
    io::{BufWriter, Write as _},
    path::PathBuf,
};

use crate::{
    cli::{Args, Scenes},
    config::{Config, Image},
};

use clap::Parser;

use scenes::{
    checkered_spheres, cornell_box, debugging_scene, lambertian_minimal, perlin_spheres, plane,
    plane_only, simple, simple_light, simple_single_light, simple_transform, simple_with_moon,
    SceneConfig, SceneGenerator,
};

mod config;
mod cli {
    use std::path::PathBuf;

    use clap::{Parser, ValueEnum};
    #[derive(Debug, Parser)]
    pub struct Args {
        #[arg(value_name = "SCENE", conflicts_with = "scene_file")]
        pub scene: Option<Scenes>,
        #[arg(long, value_name = "FILE", conflicts_with = "scene")]
        pub scene_file: Option<PathBuf>,
        #[arg(short, long, value_name = "FILE")]
        pub output: Option<PathBuf>,
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
        SimpleSingleLight,
        LambertianMinimal,
        PlaneOnly,
        SimpleWithMoon,
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
        cli::Scenes::SimpleSingleLight => &simple_single_light,
        cli::Scenes::LambertianMinimal => &lambertian_minimal,
        cli::Scenes::PlaneOnly => &plane_only,
        cli::Scenes::SimpleWithMoon => &simple_with_moon,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let config_text = read_to_string("Config.toml")?;
    let mut config = toml::from_str::<Config>(&config_text)?;
    let Image {
        aspect_ratio,
        width: image_width,
        height: image_height,
        samples_per_pixel,
        max_depth,
    } = config
        .get_image()
        .expect("Invalid aspect ratio, width, or height");

    // Scene
    let (world, lights, cam) = match (args.scene, args.scene_file.as_ref()) {
        (Some(scene), None) => get_scene_generator(scene).generate_scene(),
        (None, Some(scene_file)) => {
            let scene_text = read_to_string(scene_file)?;
            let scene_config = toml::from_str::<SceneConfig>(&scene_text).map_err(|error| {
                format!(
                    "Failed to parse scene file {}: {error}",
                    scene_file.display()
                )
            })?;
            scene_config
                .build_scene()
                .map_err(|error| format!("Invalid scene file {}: {error}", scene_file.display()))?
        }
        _ => return Err("select a built-in scene or pass --scene-file FILE".into()),
    };

    // Camera
    let cam = cam
        .with_aspect_ratio(aspect_ratio)
        .with_max_depth(max_depth)
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
    let file_name = args.output.unwrap_or(PathBuf::from("image.ppm"));
    let mut file = BufWriter::new(File::create(&file_name)?);
    file.write_fmt(format_args!("P3\n{image_width} {image_height}\n255\n"))?;
    #[cfg_attr(not(debug_assertions), expect(unused))]
    for (j, i, val) in out
        .into_iter()
        .enumerate()
        .rev()
        .flat_map(|(j, v)| v.into_iter().enumerate().map(move |(i, val)| (j, i, val)))
    {
        #[cfg(debug_assertions)]
        dbg!(j, i);

        file.write_fmt(format_args!("{val}\n"))?;
    }
    Ok(())
}
