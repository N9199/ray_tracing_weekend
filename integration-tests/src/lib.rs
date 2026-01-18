#[cfg(test)]
mod tests {
    use geometry::vec3::{Point3, Vec3};
    use scenes::{cornell_box, debugging_scene, plane, simple, simple_light};
    use shared::camera::CameraBuilder;

    #[test]
    fn plane_test() {
        // World
        let (world, lights, _) = plane();
        // dbg!(world.as_ref() as _);
        // Camera
        let cam = CameraBuilder::new()
            .with_image_width(3)
            .with_image_height(2)
            .with_samples_per_pixel(10)
            .with_max_depth(3)
            .with_lookfrom(Point3::new(-13., 2., 3.))
            .with_lookat(Point3::new(0., 0., 0.))
            .with_vup(Vec3::new(0., 1., 0.))
            .with_focus_dist(10.)
            .build();

        // Render
        cam.render_debug(world.as_ref(), lights.as_ref());
    }

    #[test]
    fn small_test() {
        // World
        let (world, lights, _) = simple();
        // dbg!(world.as_ref() as _);
        // dbg!(world.depth());
        // dbg!(world.node_count());
        // Camera
        let cam = CameraBuilder::new()
            .with_image_width(3)
            .with_image_height(2)
            .with_samples_per_pixel(10)
            .with_max_depth(3)
            .with_lookfrom(Point3::new(-13., 2., 3.))
            .with_lookat(Point3::new(0., 0., 0.))
            .with_vup(Vec3::new(0., 1., 0.))
            .with_focus_dist(10.)
            .build();

        // Render
        cam.render_debug(world.as_ref(), lights.as_ref());
    }

    #[test]
    fn small_light_test() {
        // World
        let (world, lights, _) = simple_light();
        // dbg!(world.as_ref() as _);
        // Camera
        let lookfrom = Point3::new(4., 2., 10.);
        let lookat = Point3::new(4., 2., -2.);
        let cam = CameraBuilder::new()
            .with_image_width(3)
            .with_image_height(2)
            .with_samples_per_pixel(10)
            .with_max_depth(5)
            .with_lookfrom(lookfrom)
            .with_lookat(lookat)
            .with_vup(Vec3::new(0., 1., 0.))
            .with_focus_dist(4.)
            .build();

        // Render
        cam.render_debug(world.as_ref(), lights.as_ref());
    }

    #[test]
    fn debugging_test() {
        // World
        let (world, lights, cam) = debugging_scene();
        // dbg!(world.as_ref() as _);
        // Camera
        let lookfrom = Point3::new(0., 20., 0.);
        let lookat = Point3::new(0., 0., 0.);
        let cam = cam
            .with_image_width(3)
            .with_image_height(2)
            .with_samples_per_pixel(50)
            .with_max_depth(10)
            .with_vfov(40.)
            .with_lookat(lookat)
            .with_lookfrom(lookfrom)
            .build();

        // Render
        cam.render_debug(world.as_ref(), lights.as_ref());
    }

    #[test]
    fn cornell_box_test() {
        // World
        let (world, lights, cam) = cornell_box();
        dbg!(world.as_ref(), lights.as_ref());
        // Camera
        let cam = cam
            .with_image_width(3)
            .with_image_height(2)
            .with_samples_per_pixel(50)
            .with_max_depth(10)
            .with_vfov(40.)
            .build();

        // Render
        cam.render_debug(world.as_ref(), lights.as_ref());
    }
}
