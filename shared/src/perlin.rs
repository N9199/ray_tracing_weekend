use std::ops::{Add, BitXor};

use itertools::iproduct;
use rand::{
    distributions::{Distribution, Uniform},
    thread_rng,
};

use geometry::vec3::{Point3, Vec3};

use crate::utils::random_utils::UnitSphere;

#[derive(Debug, Clone)]
pub struct Perlin {
    rand_vec: Box<[Vec3; Self::POINT_COUNT]>,
    perm_x: Box<[u8; Self::POINT_COUNT]>,
    perm_y: Box<[u8; Self::POINT_COUNT]>,
    perm_z: Box<[u8; Self::POINT_COUNT]>,
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl Perlin {
    const POINT_COUNT: usize = u8::MAX as usize + 1;

    fn perlin_generate_perm() -> Box<[u8; Self::POINT_COUNT]> {
        let mut out = Box::new([0; Self::POINT_COUNT]);
        out.iter_mut().enumerate().for_each(|(i, v)| *v = i as _);
        Self::randomize_permutation(&mut out);
        out
    }

    fn randomize_permutation(base_permutation: &mut [u8; Self::POINT_COUNT]) {
        let mut rng = thread_rng();
        (0..(Self::POINT_COUNT - 1)).for_each(|i| {
            let dist = Uniform::new(i, Self::POINT_COUNT);
            let j = dist.sample(&mut rng);
            base_permutation.swap(i, j);
        });
    }

    pub fn new() -> Self {
        let mut rng = thread_rng();
        let mut rand_vec = Box::new([Vec3::default(); Self::POINT_COUNT]);
        rand_vec
            .iter_mut()
            .for_each(|v| *v = UnitSphere.sample(&mut rng));
        Self {
            rand_vec,
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }
    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();
        // let u = u * u * (3. - 2. * u);
        // let v = v * v * (3. - 2. * v);
        // let w = w * w * (3. - 2. * w);

        let i = p.x.floor();
        let j = p.y.floor();
        let k = p.z.floor();

        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        iproduct!((0..2), (0..2), (0..2)).for_each(|(di, dj, dk)| {
            c[di][dj][dk] = self.rand_vec[self.perm_x
                [i.add(di as f64).rem_euclid(Self::POINT_COUNT as f64) as usize]
                .bitxor(self.perm_y[j.add(dj as f64).rem_euclid(Self::POINT_COUNT as f64) as usize])
                .bitxor(self.perm_z[k.add(dk as f64).rem_euclid(Self::POINT_COUNT as f64) as usize])
                as usize];
        });

        Self::perlin_interpolation(c, u, v, w)
    }

    pub fn turb(&self, p: Point3, depth: usize) -> f64 {
        let mut accum = 0.;
        let mut temp_p = p;
        let mut weight = 1.;
        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            temp_p *= 2.;
            weight *= 0.5;
        }
        accum
    }

    pub fn perlin_interpolation(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        iproduct!((0..2), (0..2), (0..2))
            .map(|(i, j, k)| {
                let temp = c[i][j][k];
                let (i, j, k) = (i as f64, j as f64, k as f64);
                let weight_v = Vec3::new(u - i, v - j, w - k);
                (i * u + (1. - i) * (1. - u))
                    * (j * v + (1. - j) * (1. - v))
                    * (k * w + (1. - k) * (1. - w))
                    * temp.dot(weight_v)
            })
            .sum()
    }

    pub fn trilinear_interpolation(c: [[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        iproduct!((0..2), (0..2), (0..2))
            .map(|(i, j, k)| {
                let temp = c[i][j][k];
                let (i, j, k) = (i as f64, j as f64, k as f64);
                (i * u + (1. - i) * (1. - u))
                    * (j * v + (1. - j) * (1. - v))
                    * (k * w + (1. - k) * (1. - w))
                    * temp
            })
            .sum()
    }
}
