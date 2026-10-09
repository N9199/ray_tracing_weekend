use std::ops::{Add, Div, Mul, Sub};
#[cfg(feature = "hit_counters")]
use std::sync::atomic::{self, AtomicU64, Ordering};

use bumpalo::Bump;
use rand::{
    Rng as _, SeedableRng as _,
    distributions::{Distribution, Standard, Uniform},
    rngs::SmallRng,
    thread_rng,
};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

use kdam::par_tqdm;

use crate::{
    colour::{Colour, SampledColour},
    hittable::{HitRecord, Hittable},
    material::ScatterReflect,
    pdf::{HittablePdf, MixturePdf, Pdf},
    ray::Ray,
    utils::random_utils::UnitDisk,
};

#[cfg(feature = "euclid")]
use geometry::vec3::Vec3Ext as _;
use geometry::vec3::{Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct CameraBuilder {
    aspect_ratio: Option<f64>,
    image_width: Option<u16>,
    image_height: Option<u16>,
    samples_per_pixel: u16,
    max_depth: u32,
    background: Colour,
    vfov: f64,
    lookfrom: Point3,
    lookat: Point3,
    vup: Vec3,
    defocus_angle: f64,
    focus_dist: f64,
}

impl CameraBuilder {
    #[must_use]
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

    #[must_use]
    pub const fn with_aspect_ratio(self, aspect_ratio: f64) -> Self {
        Self {
            aspect_ratio: Some(aspect_ratio),
            ..self
        }
    }

    #[must_use]
    pub const fn with_image_width(self, image_width: u16) -> Self {
        Self {
            image_width: Some(image_width),
            ..self
        }
    }

    #[must_use]
    pub const fn with_image_height(self, image_height: u16) -> Self {
        Self {
            image_height: Some(image_height),
            ..self
        }
    }

    #[must_use]
    pub const fn with_samples_per_pixel(self, samples_per_pixel: u16) -> Self {
        Self {
            samples_per_pixel,
            ..self
        }
    }

    #[must_use]
    pub const fn with_max_depth(self, max_depth: u32) -> Self {
        Self { max_depth, ..self }
    }

    #[must_use]
    pub const fn with_background(self, background: Colour) -> Self {
        Self { background, ..self }
    }

    #[must_use]
    pub const fn with_vfov(self, vfov: f64) -> Self {
        Self { vfov, ..self }
    }

    #[must_use]
    pub const fn with_lookfrom(self, lookfrom: Point3) -> Self {
        Self { lookfrom, ..self }
    }

    #[must_use]
    pub const fn with_lookat(self, lookat: Point3) -> Self {
        Self { lookat, ..self }
    }

    #[must_use]
    pub const fn with_vup(self, vup: Vec3) -> Self {
        Self { vup, ..self }
    }

    #[must_use]
    pub const fn with_defocus_angle(self, defocus_angle: f64) -> Self {
        Self {
            defocus_angle,
            ..self
        }
    }

    #[must_use]
    pub const fn with_focus_dist(self, focus_dist: f64) -> Self {
        Self { focus_dist, ..self }
    }

    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
                    f64::from(image_width) / f64::from(image_height),
                    image_height,
                    image_width,
                ),
                (Some(aspect_ratio), None, Some(image_width)) => (
                    aspect_ratio,
                    (f64::from(image_width) / aspect_ratio).round() as _,
                    image_width,
                ),
                (Some(aspect_ratio), Some(image_height), None) => (
                    aspect_ratio,
                    image_height,
                    (f64::from(image_height) * aspect_ratio).round() as _,
                ),
                (Some(aspect_ratio), Some(image_height), Some(image_width)) => {
                    (aspect_ratio, image_height, image_width)
                }
            };

        let pixel_samples_scale = 1. / f64::from(samples_per_pixel);
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
            .normalize()
        };
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        let viewport_u = u * viewport_width;
        let viewport_v = v * viewport_height;
        // dbg!(viewport_u, viewport_v);

        let pixel_delta_u = viewport_u / f64::from(image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

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
    #[cfg_attr(not(test), expect(unused))]
    aspect_ratio: f64,
    image_width: u16,
    image_height: u16,
    samples_per_pixel: u16,
    max_depth: u32,
    background: Colour,
    #[expect(unused)]
    vfov: f64,
    #[expect(unused)]
    lookfrom: Point3,
    #[expect(unused)]
    lookat: Point3,
    #[expect(unused)]
    vup: Vec3,
    defocus_angle: f64,
    #[expect(unused)]
    focus_dist: f64,
    #[expect(unused)]
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    #[cfg_attr(not(test), expect(unused))]
    u: Vec3,
    #[cfg_attr(not(test), expect(unused))]
    v: Vec3,
    #[cfg_attr(not(test), expect(unused))]
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum DebugModes {
    Off,
    #[cfg_attr(miri, expect(unused))]
    Normal,
    #[cfg_attr(not(miri), expect(unused))]
    Miri,
}

#[cfg(feature = "hit_counters")]
static HIT_COUNTER: AtomicU64 = AtomicU64::new(0);
impl Camera {
    // #[inline]
    pub fn get_ray(&self, i: usize, j: usize, rng: &mut dyn rand::RngCore) -> Ray {
        let dist = Uniform::new_inclusive(-0.5, 0.5);
        let offset = (dist.sample(rng), dist.sample(rng));
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset.0)
            + self.pixel_delta_v * (j as f64 + offset.1);

        debug_assert!(
            pixel_sample.x.is_finite() && pixel_sample.y.is_finite() && pixel_sample.z.is_finite()
        );

        let origin = if self.defocus_angle <= f64::EPSILON {
            self.center
        } else {
            let p = rng.sample(UnitDisk);
            self.center + self.defocus_disk_u * p.x + self.defocus_disk_v * p.z
        };
        let direction = pixel_sample - origin;
        Ray::new(origin, direction)
    }

    pub fn render(&self, world: &dyn Hittable, lights: &dyn Hittable) -> Vec<Vec<SampledColour>> {
        self.render_internal(world, lights, DebugModes::Off)
    }

    pub fn render_debug(
        &self,
        world: &dyn Hittable,
        lights: &dyn Hittable,
    ) -> Vec<Vec<SampledColour>> {
        #[cfg(not(miri))]
        const DEBUG_MODE: DebugModes = DebugModes::Normal;
        #[cfg(miri)]
        const DEBUG_MODE: DebugModes = DebugModes::Miri;

        #[cfg(debug_assertions)]
        dbg!(self);

        self.render_internal(world, lights, DEBUG_MODE)
    }

    #[inline]
    fn render_internal(
        &self,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        debug_mode: DebugModes,
    ) -> Vec<Vec<SampledColour>> {
        // Render
        let render_lambda = move |(j, i, v, mut rng, bump): (_, _, &mut Colour, _, _)| {
            *v = (0..self.samples_per_pixel)
                .map(|_| {
                    let r = self.get_ray(i, j, &mut rng);
                    Self::ray_colour_call(
                        &r,
                        &self.background,
                        world,
                        lights,
                        &mut rng,
                        &bump,
                        self.max_depth,
                    )
                })
                .fold(Colour::default(), |acc, val| acc + val);
        };
        let mut out: Vec<Vec<_>> = (0..self.image_height)
            .map(|_| (0..self.image_width).map(|_| Colour::default()).collect())
            .collect();
        let process: Vec<_> = out
            .iter_mut()
            .enumerate()
            .flat_map(|(j, vec)| {
                vec.iter_mut().enumerate().map(move |(i, v)| {
                    (
                        j,
                        i,
                        v,
                        SmallRng::from_rng(thread_rng()).unwrap(),
                        Bump::new(),
                    )
                })
            })
            .collect();

        if matches!(debug_mode, DebugModes::Miri | DebugModes::Normal) {
            process.into_iter().for_each(render_lambda);
        } else {
            par_tqdm!(process.into_par_iter()).for_each(render_lambda);
        }

        #[cfg(feature = "hit_counters")]
        {
            use crate::entities::{
                PLANE_HIT_COUNTER, QUAD_HIT_COUNTER, SPHERE_HIT_COUNTER, TRIANGLES_HIT_COUNTER,
            };
            use crate::hittable::AABBOX_HIT_COUNTER;
            use crate::material::LIGHT_HIT_COUNTER;

            let hit_counter = HIT_COUNTER.load(Ordering::Acquire);
            let aabbox_counter = AABBOX_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let triangle_counter = TRIANGLES_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let sphere_counter = SPHERE_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let quad_counter = QUAD_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let plane_counter = PLANE_HIT_COUNTER.load(atomic::Ordering::Acquire);
            let light_counter = LIGHT_HIT_COUNTER.load(atomic::Ordering::Acquire);
            dbg!(
                hit_counter,
                aabbox_counter,
                triangle_counter,
                sphere_counter,
                quad_counter,
                plane_counter,
                light_counter
            );
        }
        out.into_iter()
            .map(|vec| {
                vec.into_iter()
                    .map(|colour| SampledColour::from((colour, self.samples_per_pixel.into())))
                    .collect()
            })
            .collect()
    }

    #[allow(dead_code)]
    fn ray_colour(
        r: &Ray,
        background: &Colour,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        rng: &mut dyn rand::RngCore,
        bump: &Bump,
        depth: u32,
    ) -> Colour {
        if depth == 0 {
            return Colour::default();
        }
        let Some(rec) = world.hit(r, (f64::EPSILON)..=f64::INFINITY) else {
            return *background;
        };

        #[cfg(feature = "hit_counters")]
        HIT_COUNTER.fetch_add(1, Ordering::Relaxed);

        let colour_from_emission =
            rec.get_material()
                .emitted(rec.get_u(), rec.get_v(), rec.get_p());

        let Some(srec) = rec.get_material().scatter(r, &rec, rng, bump) else {
            return colour_from_emission;
        };

        let pdf_ptr = match srec.scatter_reflect {
            ScatterReflect::Reflect(ray) => {
                return srec.attenuation
                    * Self::ray_colour(&ray, background, world, lights, rng, bump, depth - 1);
            }
            ScatterReflect::Scatter(pdf) => pdf,
        };

        let light_pdf = HittablePdf::new(lights, rec.get_p());
        let p = MixturePdf::new(&light_pdf, pdf_ptr.as_ref());

        let scattered_ray = Ray::new(rec.get_p(), p.generate(rng));
        let pdf_value = p.value(&scattered_ray.get_direction());

        let scattering_pdf = rec.get_material().scattering_pdf(r, &rec, &scattered_ray);

        let sample_colour = Self::ray_colour(
            &scattered_ray,
            background,
            world,
            lights,
            rng,
            bump,
            depth - 1,
        )
        .fix_nan();
        let colour_from_scatter = (srec.attenuation * scattering_pdf * sample_colour) / pdf_value;
        colour_from_emission + colour_from_scatter
    }

    fn ray_colour_call(
        r: &Ray,
        background: &Colour,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        rng: &mut dyn rand::RngCore,
        bump: &Bump,
        depth: u32,
    ) -> Colour {
        Self::ray_colour_tail_call(
            r.clone(),
            background,
            world,
            lights,
            rng,
            bump,
            Colour::from_array([1., 1., 1.]),
            Colour::default(),
            depth,
            depth,
        )
    }

    #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
    fn ray_colour_tail_call(
        r: Ray,
        background: &Colour,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        rng: &mut dyn rand::RngCore,
        bump: &Bump,
        mut mult: Colour,
        res: Colour,
        depth: u32,
        max_depth: u32,
    ) -> Colour {
        if depth == 0 {
            return Colour::default() + res;
        }
        if max_depth - depth > 3 {
            let survive_prob = mult
                .into_inner()
                .to_array()
                .iter()
                .max_by(|a, b| f64::total_cmp(a, b))
                .unwrap()
                .clamp(0.05, 1.0);
            if rng.sample::<f64, _>(Standard) > survive_prob {
                return res;
            }
            mult = mult / survive_prob;
        }
        let Some(rec) = world.hit(&r, (f64::EPSILON)..=f64::INFINITY) else {
            return mult * *background + res;
        };

        #[cfg(feature = "hit_counters")]
        HIT_COUNTER.fetch_add(1, Ordering::Relaxed);

        let colour_from_emission =
            rec.get_material()
                .emitted(rec.get_u(), rec.get_v(), rec.get_p());

        let Some(srec) = rec.get_material().scatter(&r, &rec, rng, bump) else {
            return mult * colour_from_emission + res;
        };

        let (scattered_ray, pdf_value) = match srec.scatter_reflect {
            ScatterReflect::Reflect(ray) => {
                become Self::ray_colour_tail_call(
                    ray,
                    background,
                    world,
                    lights,
                    rng,
                    bump,
                    mult * srec.attenuation,
                    res,
                    depth - 1,
                    max_depth,
                );
            }
            ScatterReflect::Scatter(pdf) => {
                let rec: &HitRecord = &rec;
                let light_pdf = HittablePdf::new(lights, rec.get_p());
                let p = MixturePdf::new(&light_pdf, pdf.as_ref());
                let scattered_ray = Ray::new(rec.get_p(), p.generate(rng));
                let pdf_value = p.value(&scattered_ray.get_direction());
                (scattered_ray, pdf_value)
            }
        };

        let scattering_pdf = rec.get_material().scattering_pdf(&r, &rec, &scattered_ray);
        #[cfg(feature = "debug")]
        if pdf_value > 1000. || pdf_value == 0. || !pdf_value.is_finite() {
            eprintln!(
                "suspicious pdf_value={pdf_value} scattering_pdf={scattering_pdf} attenuation={:?}",
                srec.attenuation
            );
        }

        become Self::ray_colour_tail_call(
            scattered_ray,
            background,
            world,
            lights,
            rng,
            bump,
            mult * (srec.attenuation * scattering_pdf / pdf_value),
            res + mult * colour_from_emission,
            depth - 1,
            max_depth,
        )
    }
}

#[cfg(test)]
mod tests {
    use rand::{
        SeedableRng as _,
        distributions::{Distribution, Uniform},
        rngs::SmallRng,
    };

    use super::{Camera, CameraBuilder};
    use geometry::{
        test_utils::{assert_close, assert_point, assert_vec},
        vec3::{Point3, Vec3},
    };

    const TOLERANCE: f64 = 1e-12;

    fn build_with_dimensions(
        aspect_ratio: Option<f64>,
        width: Option<u16>,
        height: Option<u16>,
    ) -> Camera {
        let mut builder = CameraBuilder::new();
        if let Some(aspect_ratio) = aspect_ratio {
            builder = builder.with_aspect_ratio(aspect_ratio);
        }
        if let Some(width) = width {
            builder = builder.with_image_width(width);
        }
        if let Some(height) = height {
            builder = builder.with_image_height(height);
        }
        builder.build()
    }

    #[test]
    fn builder_resolves_dimension_and_aspect_combinations() {
        let wide = 16.0 / 9.0;
        let cases = [
            (None, None, None, 1.0, 100, 100),
            (None, Some(320), None, 1.0, 320, 320),
            (None, None, Some(240), 1.0, 240, 240),
            (Some(wide), None, None, wide, 100, 56),
            (None, Some(320), Some(180), 320.0 / 180.0, 320, 180),
            (Some(wide), Some(320), None, wide, 320, 180),
            (Some(wide), None, Some(180), wide, 320, 180),
            (Some(1.5), Some(100), Some(60), 1.5, 100, 60),
            (Some(wide), Some(100), Some(100), wide, 100, 100),
            (Some(2.0), Some(5), None, 2.0, 5, 3),
        ];

        for (aspect, width, height, expected_aspect, expected_width, expected_height) in cases {
            let camera = build_with_dimensions(aspect, width, height);
            assert_close(camera.aspect_ratio, expected_aspect, TOLERANCE);
            assert_eq!(camera.image_width, expected_width);
            assert_eq!(camera.image_height, expected_height);
        }
    }

    fn normal_camera() -> Camera {
        CameraBuilder::new()
            .with_lookfrom(Point3::new(0., 0., 0.))
            .with_lookat(Point3::new(0., 0., -1.))
            .with_vup(Vec3::new(0., 1., 0.))
            .with_vfov(90.)
            .with_focus_dist(10.)
            .with_aspect_ratio(16.0 / 9.0)
            .with_image_width(320)
            .with_image_height(180)
            .build()
    }

    #[test]
    fn builder_computes_normal_view_geometry() {
        let camera = normal_camera();
        let aspect = 16.0 / 9.0;
        let delta_u = 20.0 * aspect / 320.0;
        let delta_v = 20.0 / 180.0;

        assert_point(camera.center, Point3::new(0., 0., 0.), TOLERANCE);
        assert_vec(camera.u, Vec3::new(1., 0., 0.), TOLERANCE);
        assert_vec(camera.v, Vec3::new(0., 1., 0.), TOLERANCE);
        assert_vec(camera.w, Vec3::new(0., 0., 1.), TOLERANCE);
        assert_vec(camera.pixel_delta_u, Vec3::new(delta_u, 0., 0.), TOLERANCE);
        assert_vec(camera.pixel_delta_v, Vec3::new(0., delta_v, 0.), TOLERANCE);
        assert_point(
            camera.pixel00_loc,
            Point3::new(
                -20.0 * aspect / 2.0 + delta_u / 2.0,
                -20.0 / 2.0 + delta_v / 2.0,
                -10.,
            ),
            TOLERANCE,
        );
    }

    #[test]
    fn get_ray_uses_repeatable_pixel_jitter_and_normal_ray_geometry() {
        const SEED: u64 = 0xCAFE_BABE;
        const I: usize = 23;
        const J: usize = 41;
        let camera = normal_camera();
        let mut camera_rng = SmallRng::seed_from_u64(SEED);
        let ray = camera.get_ray(I, J, &mut camera_rng);

        let mut expected_rng = SmallRng::seed_from_u64(SEED);
        let jitter = Uniform::new_inclusive(-0.5, 0.5);
        let offset_x = jitter.sample(&mut expected_rng);
        let offset_y = jitter.sample(&mut expected_rng);
        let expected_direction = camera.pixel00_loc
            + camera.pixel_delta_u * (I as f64 + offset_x)
            + camera.pixel_delta_v * (J as f64 + offset_y)
            - camera.center;

        assert_point(ray.get_origin(), camera.center, TOLERANCE);
        assert_vec(ray.get_direction(), expected_direction, TOLERANCE);
    }
}
