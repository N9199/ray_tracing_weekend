use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Onb([Vec3; 3]);

impl Onb {
    #[must_use]
    pub fn new(normal: Vec3) -> Self {
        let w = normal.normalize();
        let a = if w.x.abs() > 0.9 {
            Vec3::new(0., 1., 0.)
        } else {
            Vec3::new(1., 0., 0.)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);
        Self([u, v, w])
    }

    #[must_use]
    pub const fn get_u(self) -> Vec3 {
        self.0[0]
    }
    #[must_use]
    pub const fn get_v(self) -> Vec3 {
        self.0[1]
    }
    #[must_use]
    pub const fn get_w(self) -> Vec3 {
        self.0[2]
    }
    #[must_use]
    pub fn transform(self, v: Vec3) -> Vec3 {
        (0..3).map(|i| self.0[i] * v.to_array()[i]).sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::{onb::Onb, vec3::Vec3};

    #[test]
    fn positive_x_normal() {
        let uvw = Onb::new(Vec3::from([1., 0., 0.]));
        assert!(
            uvw.get_u().to_array().iter().all(|&v| v.is_finite()),
            "u = {:?}",
            uvw.get_u()
        );
        assert!(
            uvw.get_v().to_array().iter().all(|&v| v.is_finite()),
            "v = {:?}",
            uvw.get_v()
        );
        assert!(
            uvw.get_w().to_array().iter().all(|&v| v.is_finite()),
            "w = {:?}",
            uvw.get_w()
        );
    }
}
