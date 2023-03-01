use crate::{
    ray::Ray,
    vec3::{get_in_unit_disk, Point3, Vec3},
};

#[derive(Debug, Clone)]
pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3,
        lookat: Point3,
        vup: Vec3,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Self {
        let theta = vfov.to_radians();
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (lookfrom - lookat).unit_vec();
        let u = vup.cross(w).unit_vec();
        let v = w.cross(u);

        let origin = lookfrom;
        let horizontal = u * viewport_width * focus_dist;
        let vertical = v * viewport_height * focus_dist;
        let lower_left_corner = origin - horizontal / 2. - vertical / 2. - w * focus_dist;

        let lens_radius = aperture / 2.;

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            w,
            lens_radius,
        }
    }

    #[inline]
    pub fn get_ray<T: rand::Rng>(&self, u: f64, v: f64, rng: &mut T) -> Ray {
        let rd = get_in_unit_disk(rng) * self.lens_radius;
        let offset = self.u * rd.get_x() + self.v * rd.get_y();
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + self.horizontal * u + self.vertical * v - self.origin - offset,
        )
    }
}
