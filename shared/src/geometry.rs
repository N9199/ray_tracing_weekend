pub mod matrix3;
pub mod vec3;
pub mod onb {
    use crate::geometry::vec3::Vec3;

    #[derive(Debug, Clone, Copy)]
    pub struct Onb([Vec3; 3]);

    impl Onb {
        pub fn new(normal: Vec3) -> Self {
            let w = normal.unit_vec();
            let a = if w.get_x().abs() > 0.9 {
                Vec3::new(0., 1., 0.)
            } else {
                Vec3::new(1., 0., 0.)
            };
            let v = w.cross(a).unit_vec();
            let u = w.cross(v);
            Self([u, v, w])
        }

        pub const fn get_u(self) -> Vec3 {
            self.0[0]
        }
        pub const fn get_v(self) -> Vec3 {
            self.0[1]
        }
        pub const fn get_w(self) -> Vec3 {
            self.0[2]
        }
        pub fn transform(self, v: Vec3) -> Vec3 {
            (0..3).map(|i| self.0[i] * v[i]).sum()
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::geometry::{onb::Onb, vec3::Vec3};

        #[test]
        fn positive_x_normal() {
            let uvw = Onb::new(Vec3::from([1., 0., 0.]));
            assert!(
                uvw.get_u().inner().iter().all(|&v| v.is_finite()),
                "u = {:?}",
                uvw.get_u()
            );
            assert!(
                uvw.get_v().inner().iter().all(|&v| v.is_finite()),
                "v = {:?}",
                uvw.get_v()
            );
            assert!(
                uvw.get_w().inner().iter().all(|&v| v.is_finite()),
                "w = {:?}",
                uvw.get_w()
            );
        }
    }
}
