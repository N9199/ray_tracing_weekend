#[cfg(feature = "hit_counters")]
use std::sync::atomic::AtomicU32;
use std::{f64::consts::PI, fmt::Debug, sync::Arc};

use rand::{Rng, distributions::Open01};

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

        struct IntoMaterial {
            into_material: fn(*const MaterialBytes) -> *const dyn Material,
            into_debug: fn(*const MaterialBytes) -> *const dyn Debug,
            drop_shim: fn(MaterialBytes),
            clone: fn(*const MaterialBytes) -> MaterialBytes,
        }

        trait MaterialTransform: Sized {
            const FUNCTIONS: &IntoMaterial;
        }

        impl<T> MaterialTransform for T
        where
            T: Deref<Target = dyn Material> + Debug + Clone,
        {
            const FUNCTIONS: &IntoMaterial = &IntoMaterial {
                into_material: |bytes| {
                    unsafe {
                        std::mem::transmute::<*const [std::mem::MaybeUninit<u8>; 16], *const T>(
                            bytes,
                        )
                        .as_ref()
                    }
                    .unwrap()
                    .deref() as _
                },
                into_debug: |bytes| unsafe {
                    #[cfg(debug_assertions)]
                    dbg!(std::any::type_name::<T>());
                    std::mem::transmute::<*const [std::mem::MaybeUninit<u8>; 16], *const T>(bytes)
                        .as_ref()
                        .unwrap()
                        .deref() as *const T::Target as *const dyn Debug
                },
                drop_shim: |mut bytes| unsafe {
                    if std::mem::needs_drop::<T>() {
                        let value = std::mem::transmute::<
                            *mut [std::mem::MaybeUninit<u8>; 16],
                            *mut T,
                        >(&mut bytes as *mut _);
                        drop_in_place(value);
                    }
                },
                clone: |bytes| {
                    let new_value = unsafe {
                        std::mem::transmute::<*const [std::mem::MaybeUninit<u8>; 16], *const T>(
                            bytes,
                        )
                        .as_ref()
                    }
                    .unwrap()
                    .to_owned();
                    DynMaterial::try_new(new_value).unwrap().bytes
                },
            };
        }

        pub struct DynMaterial {
            bytes: MaterialBytes,
            into_material: &'static IntoMaterial,
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
                    let material_ptr_ptr = unsafe {
                        std::mem::transmute::<*const T, *const u8>(&material_ptr as *const _)
                    };
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
                let bytes = (self.into_material.clone)(&self.bytes as _);
                Self {
                    bytes,
                    into_material: self.into_material,
                }
            }
        }

        impl Debug for DynMaterial {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let dyn_debug = unsafe {
                    (self.into_material.into_debug)(&self.bytes as _)
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
                    (self.into_material.into_material)(&self.bytes as _)
                        .as_ref()
                        .unwrap()
                }
                .scatter(ray_in, rec, rng)
            }

            #[inline]
            fn emitted(&self, u: f64, v: f64, point: Point3) -> Colour {
                unsafe {
                    (self.into_material.into_material)(&self.bytes as _)
                        .as_ref()
                        .unwrap()
                }
                .emitted(u, v, point)
            }

            #[inline]
            fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord<'_>, scattered: &Ray) -> f64 {
                unsafe {
                    (self.into_material.into_material)(&self.bytes as _)
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
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

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
    pub fn new(albedo: Colour, fuzz: f64) -> Self {
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

pub struct Dialectric {
    index_of_refraction: f64,
}

impl Debug for Dialectric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dialectric")
            .field("index_of_refraction", &self.index_of_refraction)
            .finish()
    }
}

impl Clone for Dialectric {
    fn clone(&self) -> Self {
        Self {
            index_of_refraction: self.index_of_refraction,
        }
    }
}

impl Dialectric {
    pub fn new(index_of_refraction: f64) -> Self {
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

impl Material for Dialectric {
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
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

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
    pub fn new(texture: Arc<dyn Texture>) -> Self {
        Self { texture }
    }

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
