#[cfg(feature = "hit_counters")]
use std::sync::atomic::AtomicU32;
use std::{f64::consts::PI, fmt::Debug, sync::Arc};

use rand::{distributions::Open01, Rng};

use geometry::vec3::Point3;
#[cfg(feature = "euclid")]
use geometry::vec3::Vec3Ext as _;

use crate::{
    colour::Colour,
    hittable::HitRecord,
    pdf::{CosinePdf, Pdf, SpherePdf},
    ray::Ray,
    texture::{SolidColour, Texture},
    utils::random_utils::UnitSphere,
};

#[derive(Debug)]
pub enum ScatterReflect {
    Reflect(Ray),
    Scatter(Box<dyn Pdf>),
}

#[derive(Debug)]
pub struct ScatterRecord {
    pub attenuation: Colour,
    pub scatter_reflect: ScatterReflect,
}

pub trait Material: Sync + Send + Debug {
    fn scatter(
        &self,
        _ray_in: &Ray,
        _rec: &HitRecord<'_>,
        _rng: &mut dyn rand::RngCore,
    ) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, _u: f64, _v: f64, _point: Point3) -> Colour {
        Colour::new(0., 0., 0.)
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _rec: &HitRecord<'_>, _scattered: &Ray) -> f64 {
        0.
    }
}

mod dyn_util {
    pub use dyn_enum::DynMaterial;

    // Currently UB
    #[expect(unused)]
    mod transmute {
        use std::{fmt::Debug, mem::MaybeUninit, ops::Deref, ptr::drop_in_place};

        use geometry::vec3::Point3;

        use crate::{colour::Colour, hittable::HitRecord, ray::Ray};

        use super::super::{Material, ScatterRecord};

        const MAX_SIZE: usize = 16;
        type MaterialBytes = [MaybeUninit<u8>; MAX_SIZE];

        struct VTable {
            into_material: fn(*const MaterialBytes) -> *const dyn Material,
            into_debug: fn(*const MaterialBytes) -> *const dyn Debug,
            drop_shim: fn(MaterialBytes),
            clone: fn(*const MaterialBytes) -> MaterialBytes,
        }

        trait MaterialTransform: Sized {
            const FUNCTIONS: &VTable;
        }

        impl<T> MaterialTransform for T
        where
            T: Deref<Target = dyn Material> + Debug + Clone,
        {
            const FUNCTIONS: &VTable = &VTable {
                into_material: |bytes| {
                    std::ptr::from_ref(unsafe { bytes.cast::<T>().as_ref() }.unwrap().deref())
                },
                into_debug: |bytes| unsafe {
                    #[cfg(debug_assertions)]
                    dbg!(std::any::type_name::<T>());
                    std::ptr::from_ref::<T::Target>(bytes.cast::<T>().as_ref().unwrap().deref())
                        as *const dyn Debug
                },
                drop_shim: |mut bytes| unsafe {
                    if std::mem::needs_drop::<T>() {
                        let value = (&raw mut bytes).cast::<T>();
                        drop_in_place(value);
                    }
                },
                clone: |bytes| {
                    let new_value = unsafe { bytes.cast::<T>().as_ref() }.unwrap().to_owned();
                    DynMaterial::try_new(new_value).unwrap().bytes
                },
            };
        }

        pub struct DynMaterial {
            bytes: MaterialBytes,
            into_material: &'static VTable,
        }

        impl DynMaterial {
            #[inline]
            pub fn try_new<T>(material_ptr: T) -> Option<Self>
            where
                T: Deref<Target = dyn Material> + Debug + Sized + Clone,
            {
                (size_of_val(&material_ptr) <= MAX_SIZE).then(|| {
                    let mut bytes = [MaybeUninit::zeroed(); MAX_SIZE];
                    let size = size_of::<T>();
                    let material_ptr_ptr = unsafe { (&raw const material_ptr).cast::<u8>() };
                    let material_ptr_as_slice =
                        unsafe { std::slice::from_raw_parts(material_ptr_ptr, size) };
                    bytes.iter_mut().zip(material_ptr_as_slice).for_each(
                        |(byte, material_byte)| {
                            byte.write(*material_byte);
                        },
                    );

                    #[cfg(debug_assertions)]
                    {
                        use arrayvec::ArrayVec;

                        let init_bytes: ArrayVec<_, MAX_SIZE> = bytes
                            .iter()
                            .take(size)
                            .map(|byte| unsafe { byte.assume_init_ref() })
                            .collect();
                        dbg!(std::any::type_name::<T>());
                        dbg!(init_bytes);
                    }
                    std::mem::forget(material_ptr);
                    Self {
                        bytes,
                        into_material: T::FUNCTIONS,
                    }
                })
            }

            pub(crate) fn debug_internals(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                f.debug_struct("DynMaterial")
                    .field("bytes", &self.bytes)
                    .finish()
            }
        }

        impl Clone for DynMaterial {
            fn clone(&self) -> Self {
                let bytes = (self.into_material.clone)(std::ptr::from_ref(&self.bytes));
                Self {
                    bytes,
                    into_material: self.into_material,
                }
            }
        }

        impl Debug for DynMaterial {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let dyn_debug = unsafe {
                    (self.into_material.into_debug)(std::ptr::from_ref(&self.bytes))
                        .as_ref()
                        .unwrap()
                };
                f.debug_struct("DynMaterial")
                    .field("inner", dyn_debug)
                    .finish()
            }
        }

        impl Material for DynMaterial {
            #[inline]
            fn scatter(
                &self,
                ray_in: &Ray,
                rec: &HitRecord<'_>,
                rng: &mut dyn rand::RngCore,
            ) -> Option<ScatterRecord> {
                unsafe {
                    (self.into_material.into_material)(std::ptr::from_ref(&self.bytes))
                        .as_ref()
                        .unwrap()
                }
                .scatter(ray_in, rec, rng)
            }

            #[inline]
            fn emitted(&self, u: f64, v: f64, point: Point3) -> Colour {
                unsafe {
                    (self.into_material.into_material)(std::ptr::from_ref(&self.bytes))
                        .as_ref()
                        .unwrap()
                }
                .emitted(u, v, point)
            }

            #[inline]
            fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
                unsafe {
                    (self.into_material.into_material)(std::ptr::from_ref(&self.bytes))
                        .as_ref()
                        .unwrap()
                }
                .scattering_pdf(ray_in, rec, scattered)
            }
        }
    }

    mod dyn_enum {
        use std::sync::Arc;

        use geometry::vec3::Point3;

        use crate::{colour::Colour, hittable::HitRecord, material::ScatterRecord, ray::Ray};

        use super::super::Material;

        #[derive(Debug, Clone)]
        pub enum DynMaterial {
            Ref(&'static dyn Material),
            Arc(Arc<dyn Material>),
        }

        impl DynMaterial {
            #[must_use]
            pub const fn as_bytes(&self) -> &[u8] {
                const SIZE: usize = std::mem::size_of::<DynMaterial>();
                let ptr = std::ptr::from_ref::<DynMaterial>(self).cast();
                unsafe { std::slice::from_raw_parts(ptr, SIZE) }
            }
        }

        impl TryFrom<Arc<dyn Material>> for DynMaterial {
            type Error = ();

            fn try_from(value: Arc<dyn Material>) -> Result<Self, Self::Error> {
                Ok(Self::Arc(value))
            }
        }

        impl<T: Material + 'static> TryFrom<Arc<T>> for DynMaterial {
            type Error = ();

            fn try_from(value: Arc<T>) -> Result<Self, Self::Error> {
                Ok(Self::Arc(value))
            }
        }

        impl TryFrom<&'static dyn Material> for DynMaterial {
            type Error = ();

            fn try_from(value: &'static dyn Material) -> Result<Self, Self::Error> {
                Ok(Self::Ref(value))
            }
        }

        impl Material for DynMaterial {
            fn scatter(
                &self,
                ray_in: &Ray,
                rec: &HitRecord<'_>,
                rng: &mut dyn rand::RngCore,
            ) -> Option<ScatterRecord> {
                match self {
                    DynMaterial::Ref(material) => material.scatter(ray_in, rec, rng),
                    DynMaterial::Arc(material) => material.scatter(ray_in, rec, rng),
                }
            }

            fn emitted(&self, u: f64, v: f64, point: Point3) -> Colour {
                match self {
                    DynMaterial::Ref(material) => material.emitted(u, v, point),
                    DynMaterial::Arc(material) => material.emitted(u, v, point),
                }
            }

            fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
                match self {
                    DynMaterial::Ref(material) => material.scattering_pdf(ray_in, rec, scattered),
                    DynMaterial::Arc(material) => material.scattering_pdf(ray_in, rec, scattered),
                }
            }
        }

        impl AsRef<dyn Material> for DynMaterial {
            fn as_ref<'a>(&'a self) -> &'a (dyn Material + 'static) {
                #[cfg(feature = "debug")]
                {
                    dbg!(std::any::type_name::<Self>());
                    dbg!(std::alloc::Layout::new::<Self>());
                    dbg!(self as *const _);
                    dbg!(self.as_bytes());
                    dbg!(std::mem::discriminant(self));
                    dbg!(&self);
                    // if let DynMaterial::Arc(mat) = &self {
                    //     dbg!(Arc::strong_count(&mat));
                    //     dbg!(Arc::weak_count(&mat));
                    // }
                }
                match self {
                    DynMaterial::Ref(material) => *material,
                    DynMaterial::Arc(material) => material.as_ref(),
                }
            }
        }
    }
}

pub use dyn_util::DynMaterial;

#[derive(Debug, Clone, Copy)]
pub struct Invisible;
pub const INVISIBLE_PTR: &dyn Material = &Invisible;

impl Material for Invisible {}

pub struct Lambertian {
    texture: Arc<dyn Texture>,
}

impl Debug for Lambertian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lambertian")
            .field("texture", &self.texture)
            .finish()
    }
}

impl Clone for Lambertian {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
        }
    }
}

impl Lambertian {
    #[must_use]
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    #[must_use]
    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _ray_in: &Ray,
        rec: &HitRecord<'_>,
        _rng: &mut dyn rand::RngCore,
    ) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self
                .texture
                .get_colour(rec.get_u(), rec.get_v(), rec.get_p()),
            scatter_reflect: ScatterReflect::Scatter(Box::new(CosinePdf::new(rec.get_normal()))),
        })
    }

    fn scattering_pdf(&self, _ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
        let cos_theta = rec.get_normal().dot(scattered.get_direction().normalize()) / PI;
        cos_theta.max(0.)
    }
}

pub struct Metal {
    albedo: Colour,
    fuzz: f64,
}

impl Debug for Metal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Metal")
            .field("albedo", &self.albedo)
            .field("fuzz", &self.fuzz)
            .finish()
    }
}

impl Clone for Metal {
    fn clone(&self) -> Self {
        Self {
            albedo: self.albedo,
            fuzz: self.fuzz,
        }
    }
}

impl Metal {
    #[must_use]
    pub const fn new(albedo: Colour, fuzz: f64) -> Self {
        Self { albedo, fuzz }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord<'_>,
        rng: &mut dyn rand::RngCore,
    ) -> Option<ScatterRecord> {
        let reflected = ray_in.get_direction().normalize().reflect(rec.get_normal());
        let reflected = Ray::new(rec.get_p(), reflected + rng.sample(UnitSphere) * self.fuzz);
        (reflected.get_direction().dot(rec.get_normal()) > 0.).then_some(ScatterRecord {
            attenuation: self.albedo,
            scatter_reflect: ScatterReflect::Reflect(reflected),
        })
    }
}

pub struct Dielectric {
    index_of_refraction: f64,
}

impl Debug for Dielectric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dielectric")
            .field("index_of_refraction", &self.index_of_refraction)
            .finish()
    }
}

impl Clone for Dielectric {
    fn clone(&self) -> Self {
        Self {
            index_of_refraction: self.index_of_refraction,
        }
    }
}

impl Dielectric {
    #[must_use]
    pub const fn new(index_of_refraction: f64) -> Self {
        Self {
            index_of_refraction,
        }
    }
    #[inline]
    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0 = r0 * r0;
        r0 + (1. - r0) * ((1. - cosine).powi(5))
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        ray_in: &Ray,
        rec: &HitRecord<'_>,
        rng: &mut dyn rand::RngCore,
    ) -> Option<ScatterRecord> {
        let refraction_ratio = if rec.is_front_face() {
            self.index_of_refraction.recip()
        } else {
            self.index_of_refraction
        };
        let unit_direction = ray_in.get_direction().normalize();

        let cos_theta = unit_direction.dot(-rec.get_normal()).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.;
        let direction = if cannot_refract
            || Self::reflectance(cos_theta, refraction_ratio) > rng.sample(Open01)
        {
            unit_direction.reflect(rec.get_normal())
        } else {
            unit_direction.refract(rec.get_normal(), refraction_ratio)
        };

        Some(ScatterRecord {
            attenuation: Colour::new(1., 1., 1.),
            scatter_reflect: ScatterReflect::Reflect(Ray::new(rec.get_p(), direction)),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DiffuseLight {
    texture: Arc<dyn Texture>,
}

#[cfg(feature = "hit_counters")]
pub(crate) static LIGHT_HIT_COUNTER: AtomicU32 = AtomicU32::new(0);

impl DiffuseLight {
    #[must_use]
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    #[must_use]
    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, point: Point3) -> Colour {
        #[cfg(feature = "hit_counters")]
        LIGHT_HIT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.texture.get_colour(u, v, point)
    }
}

#[derive(Debug, Clone)]
pub struct Isotropic {
    texture: Arc<dyn Texture>,
}

// static ISOTROPIC_PDF: LazyLock<Arc<SpherePdf>> = LazyLock::new(|| Arc::new(SpherePdf));
impl Isotropic {
    #[must_use]
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

    #[must_use]
    pub fn new_with_colour(colour: Colour) -> Self {
        Self::new(Arc::new(SolidColour(colour)))
    }
}

impl Material for Isotropic {
    fn scatter(
        &self,
        _ray_in: &Ray,
        rec: &HitRecord<'_>,
        _rng: &mut dyn rand::RngCore,
    ) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            attenuation: self
                .texture
                .get_colour(rec.get_u(), rec.get_v(), rec.get_p()),
            scatter_reflect: ScatterReflect::Scatter(Box::new(SpherePdf)),
        })
    }

    fn emitted(&self, _u: f64, _v: f64, _point: Point3) -> Colour {
        Colour::new(0., 0., 0.)
    }

    fn scattering_pdf(&self, _ray_in: &Ray, _rec: &HitRecord<'_>, _scattered: &Ray) -> f64 {
        1. / (4. * PI)
    }
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::PI, sync::Arc};

    use geometry::{
        test_utils::{assert_close as assert_close_with_tolerance, assert_point, assert_vec},
        vec3::{Point3, Vec3},
    };

    use rand::{distributions::Open01, rngs::SmallRng, Rng as _, SeedableRng as _};

    use crate::{
        colour::Colour,
        hittable::HitRecord,
        material::{
            Dielectric, DiffuseLight, Invisible, Isotropic, Lambertian, Material, Metal,
            ScatterReflect,
        },
        ray::Ray,
        texture::Texture,
    };

    const TOLERANCE: f64 = 1e-10;

    #[derive(Debug)]
    struct DefaultMaterial;

    impl Material for DefaultMaterial {}

    #[derive(Debug)]
    struct UvPointTexture;

    impl Texture for UvPointTexture {
        fn get_colour(&self, u: f64, v: f64, point: Point3) -> Colour {
            Colour::new(u + point.x, v + point.y, point.z)
        }
    }

    fn assert_colour(actual: Colour, expected: Colour) {
        assert_vec(actual.into_inner(), expected.into_inner(), TOLERANCE);
    }

    fn assert_close(actual: f64, expected: f64) {
        assert_close_with_tolerance(actual, expected, TOLERANCE);
    }

    fn material_record<'a>(material: &'a dyn Material, ray: &Ray) -> HitRecord<'a> {
        HitRecord::new(ray, 2., Vec3::new(0., 0., 1.), 0.23, 0.71, material)
    }

    fn seed_for_open01(predicate: impl Fn(f64) -> bool) -> u64 {
        (0..10_000)
            .find(|seed| {
                let mut rng = SmallRng::seed_from_u64(*seed);
                predicate(rng.sample(Open01))
            })
            .expect("a seed satisfying the Open01 condition should be found")
    }

    #[test]
    fn default_material_and_invisible_have_no_scatter_emission_or_scattering_density() {
        let material = DefaultMaterial;
        let ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(0., 0., -1.));
        for material in [&material as &dyn Material, &Invisible] {
            let record = material_record(material, &ray);
            let mut rng = SmallRng::seed_from_u64(0x0D3F_A017);

            assert!(material.scatter(&ray, &record, &mut rng).is_none());
            assert_colour(
                material.emitted(0.23, 0.71, record.get_p()),
                Colour::new(0., 0., 0.),
            );
            assert_close(material.scattering_pdf(&ray, &record, &ray), 0.);
        }
    }

    #[test]
    fn lambertian_forwards_hit_texture_coordinates_and_reports_cosine_density() {
        let material = Lambertian::new(Arc::new(UvPointTexture));
        let ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(0.5, -1., -2.));
        let record = material_record(&material, &ray);
        assert!(record.is_front_face());
        assert_point(record.get_p(), Point3::new(2., 0., -1.), TOLERANCE);
        let mut rng = SmallRng::seed_from_u64(0x1A2B_3C4D);
        let scatter = material.scatter(&ray, &record, &mut rng).unwrap();
        assert_colour(scatter.attenuation, Colour::new(2.23, 0.71, -1.));

        let ScatterReflect::Scatter(pdf) = scatter.scatter_reflect else {
            panic!("Lambertian should return a scattering PDF");
        };
        let normal = Vec3::new(0., 0., 1.);
        let tangent = Vec3::new(3., 0., 0.);
        let below = Vec3::new(0., 0., -2.);
        assert_close(pdf.value(&normal), 1. / PI);
        assert_close(pdf.value(&tangent), 0.);
        assert_close(pdf.value(&below), 0.);
        assert_close(
            material.scattering_pdf(&ray, &record, &Ray::new(record.get_p(), normal * 5.)),
            1. / PI,
        );
        assert_close(
            material.scattering_pdf(&ray, &record, &Ray::new(record.get_p(), tangent)),
            0.,
        );
        assert_close(
            material.scattering_pdf(&ray, &record, &Ray::new(record.get_p(), below)),
            0.,
        );
    }

    #[test]
    fn zero_fuzz_metal_reflects_exactly_and_high_fuzz_can_be_rejected() {
        let albedo = Colour::new(0.2, 0.4, 0.8);
        let ray = Ray::new(Point3::new(0., 0., 1.), Vec3::new(0., 0., -2.));
        let metal = Metal::new(albedo, 0.);
        let record = HitRecord::new(&ray, 1., Vec3::new(0., 0., 1.), 0.2, 0.7, &metal);
        let mut rng = SmallRng::seed_from_u64(0x5EED);
        let scatter = metal.scatter(&ray, &record, &mut rng).unwrap();
        assert_colour(scatter.attenuation, albedo);
        let ScatterReflect::Reflect(reflected) = scatter.scatter_reflect else {
            panic!("Metal should return a reflected ray");
        };
        assert_point(reflected.get_origin(), record.get_p(), TOLERANCE);
        assert_vec(reflected.get_direction(), Vec3::new(0., 0., 1.), TOLERANCE);

        let rough_metal = Metal::new(albedo, 1000.);
        let rough_record = HitRecord::new(&ray, 1., Vec3::new(0., 0., 1.), 0.2, 0.7, &rough_metal);
        let mut rough_rng = SmallRng::seed_from_u64(0x0BAD_5EED);
        assert!(rough_metal
            .scatter(&ray, &rough_record, &mut rough_rng)
            .is_none());
    }

    #[test]
    fn dielectric_seeded_normal_incidence_exercises_reflection_and_refraction() {
        let material = Dielectric::new(1.5);
        let ray = Ray::new(Point3::new(0., 0., 1.), Vec3::new(0., 0., -2.));
        let record = material_record(&material, &ray);
        assert!(record.is_front_face());

        let reflect_seed = seed_for_open01(|value| value < 0.04);
        let mut reflect_rng = SmallRng::seed_from_u64(reflect_seed);
        let reflected = material.scatter(&ray, &record, &mut reflect_rng).unwrap();
        assert_colour(reflected.attenuation, Colour::new(1., 1., 1.));
        let ScatterReflect::Reflect(reflected_ray) = reflected.scatter_reflect else {
            panic!("Dielectric should return a reflected ray record");
        };
        assert_vec(
            reflected_ray.get_direction(),
            Vec3::new(0., 0., 1.),
            TOLERANCE,
        );

        let refract_seed = seed_for_open01(|value| value > 0.04);
        let mut refract_rng = SmallRng::seed_from_u64(refract_seed);
        let refracted = material.scatter(&ray, &record, &mut refract_rng).unwrap();
        let ScatterReflect::Reflect(refracted_ray) = refracted.scatter_reflect else {
            panic!("Dielectric should return a reflected-ray record");
        };
        assert_vec(
            refracted_ray.get_direction(),
            Vec3::new(0., 0., -1.),
            TOLERANCE,
        );
        assert!(refracted_ray
            .get_direction()
            .to_array()
            .into_iter()
            .all(f64::is_finite));
    }

    #[test]
    fn dielectric_back_face_total_internal_reflection_is_forced() {
        let material = Dielectric::new(1.5);
        let ray = Ray::new(
            Point3::new(0., 0., 0.),
            Vec3::new(3.0_f64.sqrt() / 2., 0., 0.5),
        );
        let record = HitRecord::new(&ray, 1., Vec3::new(0., 0., 1.), 0.2, 0.7, &material);
        assert!(!record.is_front_face());
        let mut rng = SmallRng::seed_from_u64(0x71A1);
        let scatter = material.scatter(&ray, &record, &mut rng).unwrap();
        let ScatterReflect::Reflect(reflected) = scatter.scatter_reflect else {
            panic!("Total internal reflection should return a reflected ray");
        };
        assert_vec(
            reflected.get_direction(),
            Vec3::new(3.0_f64.sqrt() / 2., 0., -0.5),
            TOLERANCE,
        );
    }

    #[test]
    fn diffuse_light_emits_its_texture_and_isotropic_scatter_contract_is_constant() {
        let light = DiffuseLight::new(Arc::new(UvPointTexture));
        let ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(0.5, -1., -2.));
        let light_record = material_record(&light, &ray);
        assert_colour(
            light.emitted(0.23, 0.71, light_record.get_p()),
            Colour::new(2.23, 0.71, -1.),
        );
        let mut rng = SmallRng::seed_from_u64(0xF1A7);
        assert!(light.scatter(&ray, &light_record, &mut rng).is_none());
        assert_close(light.scattering_pdf(&ray, &light_record, &ray), 0.);

        let isotropic = Isotropic::new(Arc::new(UvPointTexture));
        let record = material_record(&isotropic, &ray);
        let scatter = isotropic.scatter(&ray, &record, &mut rng).unwrap();
        assert_colour(scatter.attenuation, Colour::new(2.23, 0.71, -1.));
        let ScatterReflect::Scatter(pdf) = scatter.scatter_reflect else {
            panic!("Isotropic should return a scattering PDF");
        };
        assert_close(pdf.value(&Vec3::new(1., 2., 3.)), 1. / (4. * PI));
        assert_close(
            isotropic.scattering_pdf(
                &ray,
                &record,
                &Ray::new(record.get_p(), Vec3::new(-4., 1., 0.5)),
            ),
            1. / (4. * PI),
        );
        assert_close(
            isotropic.scattering_pdf(
                &ray,
                &record,
                &Ray::new(record.get_p(), Vec3::new(0., 0., -9.)),
            ),
            1. / (4. * PI),
        );
        assert_colour(
            isotropic.emitted(0.23, 0.71, record.get_p()),
            Colour::new(0., 0., 0.),
        );
    }
}
