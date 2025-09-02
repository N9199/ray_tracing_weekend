pub use plane_divided::BoundedVolumeHierarchy;

mod plane_divided {
    use crate::{
        entities::{AABBox, AAPlane, Bounded},
        geometry::vec3::Vec3,
        hittable::{BoundedHittable, HitRecord, Hittable},
        hittable_collections::hittable_list::HittableList,
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
        // pub fn depth(&self) -> usize {
        //     match self {
        //         BoundedVolumeHierarchy::Leaf(_) => 1,
        //         BoundedVolumeHierarchy::Node {
        //             left,
        //             right,
        //             dividing_plane: _,
        //         } => left.depth().max(right.depth()) + 1,
        //     }
        // }

        // pub const fn node_count(&self) -> usize {
        //     match self {
        //         BoundedVolumeHierarchy::Leaf(_) => 1,
        //         BoundedVolumeHierarchy::Node {
        //             left,
        //             right,
        //             dividing_plane: _,
        //         } => left.node_count() + right.node_count() + 1,
        //     }
        // }

        // fn inner_into_iter(&self) -> impl Iterator<Item = &'_ HittableList> + '_ {
        //     match self {
        //         BoundedVolumeHierarchy::Leaf(hittable_list) => [hittable_list].into_iter(),
        //         BoundedVolumeHierarchy::Node {
        //             left,
        //             right,
        //             dividing_plane: _,
        //         } => left.inner_into_iter().chain(right.inner_into_iter()),
        //     }
        // }
    }

    impl From<HittableList> for BoundedVolumeHierarchy {
        fn from(value: HittableList) -> Self {
            if value.len() <= 5 {
                Self::Leaf(value)
            } else {
                let len = value.len();
                let (left, right, plane) = value.best_split();
                // if len == left.len() {
                //     // dbg!(plane);
                //     // dbg!("left");
                //     left.iter_bounded().for_each(|aabox| {
                //         // dbg!(aabox);
                //     });
                //     // dbg!("right");
                //     right.iter_bounded().for_each(|aabox| {
                //         // dbg!(aabox);
                //     });
                // }
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
        fn pdf_value(&self, _origin: Vec3, _direction: Vec3) -> f64 {
            0.
        }

        //TODO: Implement this using iterators
        fn random(&self, _origin: Vec3, _rng: &mut dyn rand::RngCore) -> Vec3 {
            Vec3::from([1., 0., 0.])
        }
    }

    impl BoundedHittable for BoundedVolumeHierarchy {}
}

mod flat {
    use core::iter::Iterator;
    use std::{any::TypeId, mem::MaybeUninit};

    use arrayvec::ArrayVec;

    use crate::{
        entities::{AABBox, Axis, Bounded, get_axis},
        hittable_collections::hittable_list::HittableList,
    };

    use super::super::hittable_list::RawHittableVec;

    enum BVHNode {
        Leaf {
            parent_index: usize,
            /// First is index in inner and the second is inside the RawHittableVec
            shape_index: (usize, usize),
        },
        Node {
            parent_index: u32,
            left_child_index: u32,
            right_child_index: u32,
            node_bbox: AABBox,
        },
        Root {
            left_child_index: u32,
            right_child_index: u32,
            node_bbox: AABBox,
        },
    }

    /// 16 should be big enough for now
    const CAP: usize = 16;
    pub struct BoundedVolumeHierarchy {
        inner: ArrayVec<(TypeId, RawHittableVec), CAP>,
        nodes: Vec<BVHNode>,
    }

    impl BoundedVolumeHierarchy {
        fn best_separator<'a>(
            bounded_iter: impl IntoIterator<Item = &'a dyn Bounded> + 'a,
        ) -> (Axis, usize, f64) {
            let temp_vec = bounded_iter.into_iter().map(|v| v.get_aabbox()).collect();
            fn best_separator(mut bboxes: Vec<AABBox>) -> (Axis, usize, f64) {
                // First find best axis
                let mut best_separator_val = (usize::MAX, f64::INFINITY, Axis::X, 0.);
                for axis in get_axis() {
                    bboxes.sort_unstable_by(|a, b| a.compare_by_axis(b, axis));
                    // partition_point < temp_vec.len()/2
                    // as we search for the first element which is strictly less than the element at temp_vec.len()/2
                    let partition_point = bboxes.partition_point(|v| {
                        v.axis(axis)
                            .start()
                            .total_cmp(bboxes[bboxes.len() / 2].axis(axis).start())
                            .is_lt()
                    });
                    let bbox_axis_size = bboxes.last().unwrap().axis(axis).end()
                        - bboxes.first().unwrap().axis(axis).start();
                    if (best_separator_val.0, -best_separator_val.1)
                        > (bboxes.len() - 2 * partition_point, -bbox_axis_size)
                    {
                        best_separator_val = (
                            bboxes.len() - 2 * partition_point,
                            bbox_axis_size,
                            axis,
                            *bboxes[bboxes.len() / 2].axis(axis).start(),
                        )
                    }
                }
                (
                    best_separator_val.2,
                    best_separator_val.0,
                    best_separator_val.3,
                )
            }
            best_separator(temp_vec)
        }

        fn build_nodes(
            curr_node: usize,
            i: (usize, usize),
            j: (usize, usize),
            hittables: &mut [(TypeId, RawHittableVec)],
            nodes: &mut [MaybeUninit<BVHNode>],
        ) {
            if i == j {
                nodes[curr_node].write(BVHNode::Leaf {
                    parent_index: curr_node / 2,
                    shape_index: i,
                });
                return;
            }
            // TODO: Add accum vec to disentagle actual size and positions of shit
            // Specifically for the missing take in the inner iterator
            let (a, b, c) = Self::best_separator(
                hittables
                    .iter()
                    .skip(i.0)
                    .take(j.0 - i.0)
                    .flat_map(|(_, raw_vec)| raw_vec.iter_bounded().skip(i.1)),
            );
            // Separate the first and last of the following as they are not full segments
            // hittables.iter_mut().skip(i.0).take(j.0 - i.0).for_each(|raw_vec|);
        }
    }

    impl From<HittableList> for BoundedVolumeHierarchy {
        fn from(value: HittableList) -> Self {
            assert!(value.values.len() <= CAP);
            let mut inner: ArrayVec<(TypeId, RawHittableVec), CAP> =
                value.values.into_iter().collect();
            let full_length = value.len;
            let mut nodes = Vec::with_capacity(4 * full_length);
            unsafe { nodes.set_len(4 * full_length) };

            Self::build_nodes(
                0,
                (0, 0),
                (inner.len() - 1, inner[inner.len() - 1].1.len()),
                inner.as_mut_slice(),
                nodes.as_mut_slice(),
            );

            let nodes = {
                let ptr = nodes.as_mut_ptr().cast();
                let cap = nodes.capacity();
                let len = nodes.len();
                std::mem::forget(nodes);
                unsafe { Vec::from_raw_parts(ptr, len, cap) }
            };
            Self { inner, nodes }
        }
    }
}
