use core::f64;
use std::{
    fs::File,
    io::{BufWriter, Write as _},
    ops::{Add, Div, Mul, Sub},
    path::Path,
    sync::{
        atomic::{self, AtomicU64, Ordering},
        Arc,
    },
};

use itertools::iproduct;
use kdam::par_tqdm;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    thread_rng, SeedableRng as _,
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

#[cfg(debug_assertions)]
use crate::entities::{AABOX_HIT_COUNTER, PLANE_HIT_COUNTER, QUAD_HIT_COUNTER, SPHERE_HIT_COUNTER};
use crate::{
    geometry::vec3::{Colour, Point3, SampledColour, Vec3},
    hittable::Hittable,
    material::ScatterReflect,
    pdf::{HittablePdf, MixturePdf, Pdf},
    ray::Ray,
    utils::random_utils::UnitDisk,
};

#[derive(Debug, Clone, Copy)]
pub struct CameraBuilder {
    aspect_ratio: Option<f64>,
    image_width: Option<u32>,
    image_height: Option<u32>,
    samples_per_pixel: u16,
    max_depth: u32,
    background: Colour,
    vfov: f64,
    lookfrom: Point3,
    lookat: Point3,
    vup: Point3,
    defocus_angle: f64,
    focus_dist: f64,
}

impl CameraBuilder {
    pub const fn new() -> Self {
        Self {
            aspect_ratio: None,
            image_width: None,
            image_height: None,
            samples_per_pixel: 10,
            max_depth: 10,
            background: Colour::new(0., 0., 0.),
            vfov: 90.,
            lookfrom: Point3::new(0., 0., 0.),
            lookat: Point3::new(0., 0., -1.),
            vup: Vec3::new(0., 1., 0.),
            defocus_angle: 0.,
            focus_dist: 10.,
        }
    }

    pub const fn with_aspect_ratio(self, aspect_ratio: f64) -> Self {
        Self {
            aspect_ratio: Some(aspect_ratio),
            ..self
        }
    }
    pub const fn with_image_width(self, image_width: u32) -> Self {
        Self {
            image_width: Some(image_width),
            ..self
        }
    }
    pub const fn with_image_height(self, image_height: u32) -> Self {
        Self {
            image_height: Some(image_height),
            ..self
        }
    }
    pub const fn with_samples_per_pixel(self, samples_per_pixel: u16) -> Self {
        Self {
            samples_per_pixel,
            ..self
        }
    }
    pub const fn with_max_depth(self, max_depth: u32) -> Self {
        Self { max_depth, ..self }
    }
    pub const fn with_background(self, background: Colour) -> Self {
        Self { background, ..self }
    }
    pub const fn with_vfov(self, vfov: f64) -> Self {
        Self { vfov, ..self }
    }
    pub const fn with_lookfrom(self, lookfrom: Point3) -> Self {
        Self { lookfrom, ..self }
    }
    pub const fn with_lookat(self, lookat: Point3) -> Self {
        Self { lookat, ..self }
    }
    pub const fn with_vup(self, vup: Point3) -> Self {
        Self { vup, ..self }
    }
    pub const fn with_defocus_angle(self, defocus_angle: f64) -> Self {
        Self {
            defocus_angle,
            ..self
        }
    }
    pub const fn with_focus_dist(self, focus_dist: f64) -> Self {
        Self { focus_dist, ..self }
    }

    pub fn build(self) -> Camera {
        let CameraBuilder {
            aspect_ratio,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            background,
            vfov,
            lookfrom,
            lookat,
            vup,
            defocus_angle,
            focus_dist,
        } = self;

        let (aspect_ratio, image_height, image_width) =
            match (aspect_ratio, image_height, image_width) {
                (None, None, None) => (1., 100, 100),
                (None, None, Some(image_width)) => (1., image_width, image_width),
                (None, Some(image_height), None) => (1., image_height, image_height),
                (Some(aspect_ratio), None, None) => {
                    (aspect_ratio, (100. / aspect_ratio).round() as _, 100)
                }
                (None, Some(image_height), Some(image_width)) => (
                    image_width as f64 / image_height as f64,
                    image_height,
                    image_width,
                ),
                (Some(aspect_ratio), None, Some(image_width)) => (
                    aspect_ratio,
                    (image_width as f64 / aspect_ratio).round() as _,
                    image_width,
                ),
                (Some(aspect_ratio), Some(image_height), None) => (
                    aspect_ratio,
                    image_height,
                    (image_height as f64 * aspect_ratio).round() as _,
                ),
                (Some(aspect_ratio), Some(image_height), Some(image_width)) => {
                    (aspect_ratio, image_height, image_width)
                }
            };

        let pixel_samples_scale = 1. / samples_per_pixel as f64;
        let center = lookfrom;

        let theta = vfov.to_radians();
        let h = theta.div(2.).tan();
        let viewport_height = 2. * h * focus_dist;
        let viewport_width = viewport_height * aspect_ratio;

        let w = {
            let w = lookfrom.sub(lookat);
            if vup.cross(w).is_near_zero() {
                // TODO Better handling of this case
                w.add(Vec3::new(0.1, 0., 0.))
            } else {
                w
            }
            .unit_vec()
        };
        let u = vup.cross(w).unit_vec();
        let v = w.cross(u);

        let viewport_u = u * viewport_width;
        let viewport_v = v * viewport_height;
        // dbg!(viewport_u, viewport_v);

        let pixel_delta_u = viewport_u / (image_width as _);
        let pixel_delta_v = viewport_v / (image_height as _);

        let viewport_upper_left_corner =
            center - (w * focus_dist) - viewport_u / 2. - viewport_v / 2.;
        let pixel00_loc = viewport_upper_left_corner + (pixel_delta_u + pixel_delta_v) / 2.;
        // dbg!(viewport_upper_left_corner, pixel00_loc);
        let defocus_radius = defocus_angle.div(2.).tan().mul(focus_dist);
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            aspect_ratio,
            image_width,
            image_height,
            samples_per_pixel,
            max_depth,
            background,
            vfov,
            lookfrom,
            lookat,
            vup,
            defocus_angle,
            focus_dist,
            pixel_samples_scale,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            u,
            v,
            w,
            defocus_disk_u,
            defocus_disk_v,
        }
    }
}

impl Default for CameraBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    samples_per_pixel: u16,
    max_depth: u32,
    background: Colour,
    vfov: f64,
    lookfrom: Point3,
    lookat: Point3,
    vup: Point3,
    defocus_angle: f64,
    focus_dist: f64,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

pub(crate) enum DebugModes {
    Off,
    Normal,
    Miri,
}

static HIT_COUNTER: AtomicU64 = AtomicU64::new(0);
impl Camera {
    // #[inline]
    pub fn get_ray<T: rand::Rng + ?Sized>(&self, i: usize, j: usize, rng: &mut T) -> Ray {
        let dist = Uniform::new_inclusive(-0.5, 0.5);
        let offset = (dist.sample(rng), dist.sample(rng));
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset.0)
            + self.pixel_delta_v * (j as f64 + offset.1);

        debug_assert!(
            pixel_sample.get_x().is_finite()
                && pixel_sample.get_y().is_finite()
                && pixel_sample.get_z().is_finite()
        );

        let origin = if self.defocus_angle <= f64::EPSILON {
            self.center
        } else {
            let p = rng.sample(UnitDisk);
            self.center + self.defocus_disk_u * p.get_x() + self.defocus_disk_v * p.get_z()
        };
        let direction = pixel_sample - origin;
        // dbg!(offset, pixel_sample, origin, direction);
        Ray::new(origin, direction)
    }

    pub fn render<P>(&self, world: &dyn Hittable, lights: &dyn Hittable, file_name: Option<P>)
    where
        P: AsRef<Path>,
    {
        let default = Path::new("image.ppm");
        let file_name = match file_name.as_ref() {
            Some(file_name) => file_name.as_ref(),
            None => default,
        };
        self.render_internal(world, lights, file_name, DebugModes::Off);
    }

    pub fn render_debug<P>(&self, world: &dyn Hittable, lights: &dyn Hittable, file_name: Option<P>)
    where
        P: AsRef<Path>,
    {
        let default = Path::new("image.ppm");
        let file_name = match file_name.as_ref() {
            Some(file_name) => file_name.as_ref(),
            None => default,
        };
        #[cfg(debug_assertions)]
        dbg!(self);

        #[cfg(not(miri))]
        const DEBUG_MODE: DebugModes = DebugModes::Normal;
        #[cfg(miri)]
        const DEBUG_MODE: DebugModes = DebugModes::Miri;
        self.render_internal(world, lights, file_name, DEBUG_MODE);
    }

    fn render_internal(
        &self,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        file_name: &Path,
        debug_mode: DebugModes,
    ) {
        // Render
        let mut out: Vec<Vec<_>> = (0..self.image_height)
            .map(|_| (0..self.image_width).map(|_| Colour::default()).collect())
            .collect();
        let lambda = |(j, i, v): (_, _, &mut Colour)| {
            *v = (0..self.samples_per_pixel)
                .map(|_| {
                    let mut rng = SmallRng::from_rng(thread_rng()).unwrap();
                    let r = self.get_ray(i, j, &mut rng);
                    self.ray_colour(&r, world, lights, &mut rng, self.max_depth)
                })
                .fold(Colour::default(), |acc, val| acc + val);
        };
        let process: Vec<_> = out
            .iter_mut()
            .enumerate()
            .flat_map(|(j, vec)| vec.iter_mut().enumerate().map(move |(i, v)| (j, i, v)))
            .collect();
        if matches!(debug_mode, DebugModes::Miri | DebugModes::Normal) {
            process.into_iter().for_each(lambda);
        } else {
            par_tqdm!(process.into_par_iter()).for_each(lambda);
        }

        if matches!(debug_mode, DebugModes::Miri) {
            return;
        }
        let mut file = BufWriter::new(File::create(file_name).unwrap());
        file.write_fmt(format_args!(
            "P3\n{} {}\n255\n",
            self.image_width, self.image_height
        ))
        .unwrap();
        for (j, i) in iproduct!((0..self.image_height).rev(), 0..self.image_width) {
            file.write_fmt(format_args!(
                "{}\n",
                SampledColour::from((out[j as usize][i as usize], self.samples_per_pixel as _))
            ))
            .unwrap();
        }
        #[cfg(debug_assertions)]
        {
            use crate::material::LIGHT_HIT_COUNTER;

            let hit_counter = HIT_COUNTER.load(Ordering::Acquire);
            let aabox_counter = AABOX_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let sphere_counter = SPHERE_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let quad_counter = QUAD_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let plane_counter = PLANE_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let light_counter = LIGHT_HIT_COUNTER.load(atomic::Ordering::Acquire);
            dbg!(
                hit_counter,
                aabox_counter,
                sphere_counter,
                quad_counter,
                plane_counter,
                light_counter
            );
        }
    }

    pub fn ray_colour(
        &self,
        r: &Ray,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        rng: &mut dyn rand::RngCore,
        depth: u32,
    ) -> Colour {
        if depth == 0 {
            return Colour::default();
        }
        let Some(rec) = world.hit(r, (f64::EPSILON)..=f64::INFINITY) else {
            return self.background;
        };
        HIT_COUNTER.fetch_add(1, Ordering::Relaxed);

        let colour_from_emission =
            rec.get_material()
                .emitted(rec.get_u(), rec.get_v(), rec.get_p());

        let Some(srec) = rec.get_material().scatter(r, &rec) else {
            return colour_from_emission;
        };

        let pdf_ptr = match srec.scatter_reflect {
            ScatterReflect::Reflect(ray) => {
                return srec.attenuation * self.ray_colour(&ray, world, lights, rng, depth - 1);
            }
            ScatterReflect::Scatter(pdf) => pdf,
        };

        let light_pdf = HittablePdf::new(lights, rec.get_p());
        let p = MixturePdf::new(&light_pdf, pdf_ptr.as_ref());

        let scattered_ray = Ray::new(rec.get_p(), p.generate(rng));
        let pdf_value = p.value(&scattered_ray.get_direction());

        let scattering_pdf = rec.get_material().scattering_pdf(r, &rec, &scattered_ray);

        let sample_colour = self
            .ray_colour(&scattered_ray, world, lights, rng, depth - 1)
            .fix_nan();
        let colour_from_scatter = (srec.attenuation * scattering_pdf * sample_colour) / pdf_value;
        colour_from_emission + colour_from_scatter
    }
}
