pub use plane_divided::BoundedVolumeHierarchy;

mod plane_divided {
    use crate::{
        entities::{AABBox, AAPlane, Bounded},
        geometry::vec3::Vec3,
        hittable::{BoundedHittable, HitRecord, Hittable},
        hittable_list::HittableList,
        ray::Ray,
    };

    // TODO Implement as list recursively sorted
    // TODO Implement as BTree
    // See https://doc.rust-lang.org/src/alloc/collections/btree/node.rs.html
    #[derive(Debug)]
    pub enum BoundedVolumeHierarchy {
        Leaf(HittableList),
        Node {
            left: Box<BoundedVolumeHierarchy>,
            right: Box<BoundedVolumeHierarchy>,
            dividing_plane: AAPlane,
        },
    }

    impl BoundedVolumeHierarchy {
        pub fn depth(&self) -> usize {
            match self {
                BoundedVolumeHierarchy::Leaf(_) => 1,
                BoundedVolumeHierarchy::Node {
                    left,
                    right,
                    dividing_plane: _,
                } => left.depth().max(right.depth()) + 1,
            }
        }

        pub const fn node_count(&self) -> usize {
            match self {
                BoundedVolumeHierarchy::Leaf(_) => 1,
                BoundedVolumeHierarchy::Node {
                    left,
                    right,
                    dividing_plane: _,
                } => left.node_count() + right.node_count() + 1,
            }
        }
    }

    impl From<HittableList> for BoundedVolumeHierarchy {
        fn from(value: HittableList) -> Self {
            if value.len() <= 5 {
                Self::Leaf(value)
            } else {
                let len = value.len();
                let (left, right, plane) = value.best_split();
                if len == left.len() {
                    // dbg!(plane);
                    // dbg!("left");
                    left.iter_bounded().for_each(|aabox| {
                        // dbg!(aabox);
                    });
                    // dbg!("right");
                    right.iter_bounded().for_each(|aabox| {
                        // dbg!(aabox);
                    });
                }
                debug_assert_ne!(len, left.len());
                debug_assert_ne!(len, right.len());
                if len == left.len() {
                    Self::Leaf(left)
                } else if len == right.len() {
                    Self::Leaf(right)
                } else {
                    Self::Node {
                        left: Box::new(left.into()),
                        right: Box::new(right.into()),
                        dividing_plane: plane,
                    }
                }
            }
        }
    }

    impl Bounded for BoundedVolumeHierarchy {
        fn get_aabbox(&self) -> AABBox {
            match self {
                Self::Leaf(value) => value.get_aabbox(),
                Self::Node {
                    left,
                    right,
                    dividing_plane: _,
                } => left.get_aabbox().enclose(&right.get_aabbox()),
            }
        }

        fn get_surface_area(&self) -> f64 {
            match self {
                BoundedVolumeHierarchy::Leaf(value) => value.get_surface_area(),
                BoundedVolumeHierarchy::Node {
                    left,
                    right,
                    dividing_plane: _,
                } => left.get_surface_area() + right.get_surface_area(),
            }
        }
    }

    impl Hittable for BoundedVolumeHierarchy {
        fn hit(&self, r: &Ray, range: std::ops::RangeInclusive<f64>) -> Option<HitRecord<'_>> {
            match self {
                BoundedVolumeHierarchy::Leaf(list) => list.hit(r, range),
                BoundedVolumeHierarchy::Node {
                    left,
                    right,
                    dividing_plane: _,
                } => {
                    match (
                        left.get_aabbox()
                            .is_hit(r, range.clone())
                            .then(|| left.hit(r, range.clone()))
                            .flatten(),
                        right
                            .get_aabbox()
                            .is_hit(r, range.clone())
                            .then(|| right.hit(r, range.clone()))
                            .flatten(),
                    ) {
                        (None, None) => None,
                        (None, Some(v)) => Some(v),
                        (Some(v), None) => Some(v),
                        (Some(v1), Some(v2)) => [v1, v2]
                            .into_iter()
                            .min_by(|a, b| a.get_t().total_cmp(&b.get_t())),
                    }
                }
            }
        }

        //TODO: Implement this using iterators
        fn pdf_value(&self, origin: Vec3, direction: Vec3) -> f64 {
            0.
        }

        //TODO: Implement this using iterators
        fn random(&self, origin: Vec3, rng: &mut dyn rand::RngCore) -> Vec3 {
            Vec3::from([1., 0., 0.])
        }
    }

    impl BoundedHittable for BoundedVolumeHierarchy {}

    //TODO: Implement this
    mod iter {}
}

mod sorted_divided {
    use crate::hittable_list::HittableList;

    pub struct BoundedVolumeHierarchy {
        entitites: HittableList,
    }
}
