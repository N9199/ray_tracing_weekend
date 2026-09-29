use std::sync::Arc;

#[cfg(feature = "euclid")]
use geometry::aabbox::Box3DExt as _;
use geometry::{
    aabbox::AABBox,
    aaplane::get_axis,
    bounded::Bounded,
    test_utils::{assert_close_scaled, assert_point_scaled, assert_vec_scaled},
    vec3::{Point3, Vec3},
};
use shared::{
    colour::Colour,
    entities::{Cuboid, Quad, Sphere},
    hittable::{HitRecord, Hittable},
    hittable_collections::{bvh::BoundedVolumeHierarchy, hittable_list::HittableList},
    material::Lambertian,
    ray::Ray,
};

const TOLERANCE: f64 = 1.0e-9;

#[derive(Clone, Copy)]
enum ObjectSpec {
    Sphere { center: Point3, radius: f64 },
    Quad { q: Point3, u: Vec3, v: Vec3 },
    Cuboid { min: Point3, max: Point3 },
}

fn build_list(specs: &[ObjectSpec]) -> HittableList {
    let material = Arc::new(Lambertian::new_with_colour(Colour::new(0.7, 0.4, 0.2)));
    let mut list = HittableList::default();
    for spec in specs {
        match *spec {
            ObjectSpec::Sphere { center, radius } => {
                list.add(Sphere::new(center, radius, material.clone()));
            }
            ObjectSpec::Quad { q, u, v } => {
                list.add(Quad::new(q, u, v, material.clone()));
            }
            ObjectSpec::Cuboid { min, max } => {
                list.add(Cuboid::new(min, max, material.clone()));
            }
        }
    }
    list
}

fn small_spheres() -> Vec<ObjectSpec> {
    [(-4., -2., 0., 0.75), (0., 1., 4., 0.8), (4., -1., 8., 0.7)]
        .into_iter()
        .map(|(x, y, z, radius)| ObjectSpec::Sphere {
            center: Point3::new(x, y, z),
            radius,
        })
        .collect()
}

fn six_plus_spheres() -> Vec<ObjectSpec> {
    [
        (-2., -1., -16., 0.7),
        (2., 1., -12., 0.75),
        (-1.5, 1.5, -8., 0.65),
        (0., 0., 0., 0.8),
        (2.5, 2.5, 2., 0.2),
        (0.25, 0.2, 4., 0.9),
        (1.5, -1.5, 12., 0.7),
        (2., -2., 16., 0.85),
    ]
    .into_iter()
    .map(|(x, y, z, radius)| ObjectSpec::Sphere {
        center: Point3::new(x, y, z),
        radius,
    })
    .collect()
}

fn heterogeneous_fixture() -> Vec<ObjectSpec> {
    vec![
        ObjectSpec::Sphere {
            center: Point3::new(-7., -2., 4.),
            radius: 0.8,
        },
        ObjectSpec::Sphere {
            center: Point3::new(7., 2., 8.),
            radius: 0.9,
        },
        ObjectSpec::Sphere {
            center: Point3::new(-5., 5., 12.),
            radius: 0.7,
        },
        ObjectSpec::Sphere {
            center: Point3::new(5., -5., 16.),
            radius: 0.75,
        },
        ObjectSpec::Quad {
            q: Point3::new(-2., -2., 0.),
            u: Vec3::new(3., 0., 0.),
            v: Vec3::new(0., 3., 0.),
        },
        ObjectSpec::Cuboid {
            min: Point3::new(2., -1., 1.),
            max: Point3::new(4., 1., 3.),
        },
    ]
}

fn center_ray(center: Point3) -> Ray {
    let direction = Vec3::new(1., 2., 2.);
    Ray::new(center - direction * 3., direction)
}

struct RayCase {
    label: &'static str,
    ray: Ray,
    range: std::ops::RangeInclusive<f64>,
    expected_hit: bool,
}

fn assert_close(actual: f64, expected: f64) {
    assert_close_scaled(actual, expected, TOLERANCE);
}

fn assert_vec_close(actual: Vec3, expected: Vec3) {
    assert_vec_scaled(actual, expected, TOLERANCE);
}

fn assert_point_close(actual: Point3, expected: Point3) {
    assert_point_scaled(actual, expected, TOLERANCE);
}

fn assert_hit_records_close(actual: &HitRecord<'_>, expected: &HitRecord<'_>) {
    assert_close(actual.get_t(), expected.get_t());
    assert_point_close(actual.get_p(), expected.get_p());
    assert_vec_close(actual.get_normal(), expected.get_normal());
    assert_close(actual.get_u(), expected.get_u());
    assert_close(actual.get_v(), expected.get_v());
    assert_eq!(actual.is_front_face(), expected.is_front_face());
}

fn compare_cases(reference: &HittableList, bvh: &BoundedVolumeHierarchy, cases: Vec<RayCase>) {
    for case in cases {
        let reference_hit = reference.hit(&case.ray, case.range.clone());
        let bvh_hit = bvh.hit(&case.ray, case.range);
        assert_eq!(
            reference_hit.is_some(),
            case.expected_hit,
            "reference result for {}",
            case.label
        );
        assert_eq!(
            bvh_hit.is_some(),
            case.expected_hit,
            "BVH result for {}",
            case.label
        );
        if let (Some(reference_hit), Some(bvh_hit)) = (reference_hit, bvh_hit) {
            assert_hit_records_close(&bvh_hit, &reference_hit);
        }
    }
}

fn sphere_cases(center: Point3, radius: f64, label: &'static str) -> Vec<RayCase> {
    let forward = center_ray(center);
    let reverse_direction = Vec3::new(-1., -2., -2.);
    let reverse = Ray::new(center - reverse_direction * 3., reverse_direction);
    let offset = Vec3::new(3., 2., 4.);
    let off_target = Ray::new(center + offset, -offset);
    let exact_front_parameter = 3. - radius / 3.;

    vec![
        RayCase {
            label,
            ray: forward.clone(),
            range: 0. ..=10.,
            expected_hit: true,
        },
        RayCase {
            label: "sphere hit from the reverse side",
            ray: reverse,
            range: 0. ..=10.,
            expected_hit: true,
        },
        RayCase {
            label: "sphere hit at an inclusive range endpoint",
            ray: forward,
            range: exact_front_parameter..=exact_front_parameter,
            expected_hit: true,
        },
        RayCase {
            label: "sphere hit excluded by the range",
            ray: off_target.clone(),
            range: 0. ..=0.5,
            expected_hit: false,
        },
        RayCase {
            label: "sphere hit admitted by the range",
            ray: off_target,
            range: 0. ..=2.,
            expected_hit: true,
        },
        RayCase {
            label: "finite ray misses all fixture objects",
            ray: Ray::new(Point3::new(100., 70., -40.), Vec3::new(-1., 0.3, 0.2)),
            range: 0. ..=10.,
            expected_hit: false,
        },
    ]
}

fn axis_values(bounds: AABBox) -> [f64; 6] {
    let mut values = [0.; 6];
    for (index, axis) in get_axis().into_iter().enumerate() {
        let interval = bounds.axis(axis);
        values[index * 2] = *interval.start();
        values[index * 2 + 1] = *interval.end();
    }
    values
}

fn assert_bounds_enclose(outer: AABBox, inner: AABBox) {
    for axis in get_axis() {
        let outer_axis = outer.axis(axis);
        let inner_axis = inner.axis(axis);
        assert!(*outer_axis.start() <= *inner_axis.start() + TOLERANCE);
        assert!(*outer_axis.end() + TOLERANCE >= *inner_axis.end());
    }
}

fn bounds_match(left: [f64; 6], right: [f64; 6]) -> bool {
    left.into_iter().zip(right).all(|(a, b)| {
        let allowance = TOLERANCE * a.abs().max(b.abs()).max(1.);
        (a - b).abs() <= allowance
    })
}

fn assert_same_bounds_multiset(mut expected: Vec<[f64; 6]>, actual: Vec<[f64; 6]>) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "different bounded-object counts"
    );
    for actual_bounds in actual {
        let Some(index) = expected
            .iter()
            .position(|expected_bounds| bounds_match(*expected_bounds, actual_bounds))
        else {
            panic!("BVH/split contains an unexpected object bound: {actual_bounds:?}");
        };
        expected.swap_remove(index);
    }
}

fn list_bounds(list: &HittableList) -> Vec<[f64; 6]> {
    list.iter_bounded()
        .map(|object| axis_values(object.get_aabbox()))
        .collect()
}

fn collect_bvh_bounds(bvh: &BoundedVolumeHierarchy, out: &mut Vec<[f64; 6]>) {
    match bvh {
        BoundedVolumeHierarchy::Leaf(list) => out.extend(list_bounds(list)),
        BoundedVolumeHierarchy::Node { left, right, .. } => {
            collect_bvh_bounds(left, out);
            collect_bvh_bounds(right, out);
        }
    }
}

fn assert_tree_conservation(bvh: &BoundedVolumeHierarchy) {
    match bvh {
        BoundedVolumeHierarchy::Leaf(list) => {
            assert_eq!(bvh.len(), list.len());
            assert_eq!(list.len(), list.iter_bounded().count());
            for bounds in list.iter_bounded().map(Bounded::get_aabbox) {
                assert_bounds_enclose(bvh.get_aabbox(), bounds);
            }
        }
        BoundedVolumeHierarchy::Node {
            left, right, len, ..
        } => {
            assert!(!left.is_empty());
            assert!(!right.is_empty());
            assert_eq!(*len, left.len() + right.len());
            assert_eq!(bvh.len(), *len);
            assert_bounds_enclose(bvh.get_aabbox(), left.get_aabbox());
            assert_bounds_enclose(bvh.get_aabbox(), right.get_aabbox());
            assert_tree_conservation(left);
            assert_tree_conservation(right);
        }
    }
}

fn assert_fixture_bounds_and_conservation(specs: &[ObjectSpec]) {
    let source = build_list(specs);
    let source_count = source.len();
    let source_bounds = list_bounds(&source);
    let aggregate = source.get_aabbox();
    for object in source.iter_bounded() {
        assert_bounds_enclose(aggregate, object.get_aabbox());
    }

    let (first, second, _) = source.best_split();
    assert!(!first.is_empty());
    assert!(!second.is_empty());
    assert_eq!(first.len() + second.len(), source_count);
    let mut split_bounds = list_bounds(&first);
    split_bounds.extend(list_bounds(&second));
    assert_same_bounds_multiset(source_bounds.clone(), split_bounds);

    let reference = build_list(specs);
    let bvh = BoundedVolumeHierarchy::from(build_list(specs));
    assert_eq!(bvh.len(), source_count);
    assert_tree_conservation(&bvh);
    assert_bounds_enclose(bvh.get_aabbox(), aggregate);
    assert_bounds_enclose(aggregate, bvh.get_aabbox());
    let mut bvh_bounds = Vec::new();
    collect_bvh_bounds(&bvh, &mut bvh_bounds);
    assert_same_bounds_multiset(source_bounds, bvh_bounds);
    assert_close(bvh.get_surface_area(), reference.get_surface_area());
}

#[test]
fn empty_and_leaf_threshold_behavior_matches_hittable_list() {
    let empty_reference = build_list(&[]);
    let empty_bvh = BoundedVolumeHierarchy::from(build_list(&[]));
    assert_eq!(empty_bvh.len(), empty_reference.len());
    assert_eq!(empty_bvh.is_empty(), empty_reference.is_empty());
    assert!(matches!(empty_bvh, BoundedVolumeHierarchy::Leaf(_)));
    let empty_ray = Ray::new(Point3::new(1., 2., 3.), Vec3::new(-1., -2., -3.));
    assert!(empty_reference.hit(&empty_ray, 0. ..=10.).is_none());
    assert!(empty_bvh.hit(&empty_ray, 0. ..=10.).is_none());

    for count in 1..=6 {
        let specs = (0..count)
            .map(|index| ObjectSpec::Sphere {
                center: Point3::new(
                    -8. + index as f64 * 3.,
                    -2. + (index % 2) as f64 * 4.,
                    index as f64 * 2.,
                ),
                radius: 0.6,
            })
            .collect::<Vec<_>>();
        let reference = build_list(&specs);
        let bvh = BoundedVolumeHierarchy::from(build_list(&specs));
        assert_eq!(bvh.len(), count);
        assert!(!bvh.is_empty());
        if count <= 5 {
            assert!(matches!(bvh, BoundedVolumeHierarchy::Leaf(_)));
        } else {
            assert!(matches!(bvh, BoundedVolumeHierarchy::Node { .. }));
        }

        let first_center = Point3::new(-8., -2., 0.);
        let hit_ray = center_ray(first_center);
        assert!(reference.hit(&hit_ray, 0. ..=10.).is_some());
        assert!(bvh.hit(&hit_ray, 0. ..=10.).is_some());
        let miss_ray = Ray::new(Point3::new(100., 70., -40.), Vec3::new(-1., 0.3, 0.2));
        assert!(reference.hit(&miss_ray, 0. ..=10.).is_none());
        assert!(bvh.hit(&miss_ray, 0. ..=10.).is_none());
    }
}

#[test]
fn bvh_and_reference_match_hit_records_for_finite_fixtures() {
    let small = small_spheres();
    let small_reference = build_list(&small);
    let small_bvh = BoundedVolumeHierarchy::from(build_list(&small));
    compare_cases(
        &small_reference,
        &small_bvh,
        sphere_cases(Point3::new(-4., -2., 0.), 0.75, "small fixture sphere hit"),
    );

    let separated = six_plus_spheres();
    let separated_reference = build_list(&separated);
    let separated_bvh = BoundedVolumeHierarchy::from(build_list(&separated));
    compare_cases(
        &separated_reference,
        &separated_bvh,
        sphere_cases(Point3::new(0., 0., 0.), 0.8, "six-plus fixture sphere hit"),
    );

    let heterogeneous = heterogeneous_fixture();
    let heterogeneous_reference = build_list(&heterogeneous);
    let heterogeneous_bvh = BoundedVolumeHierarchy::from(build_list(&heterogeneous));
    let quad_point = Point3::new(-0.5, -0.2, 0.);
    let quad_ray = Ray::new(
        quad_point + Vec3::new(2., 1., -4.),
        Vec3::new(-0.5, -0.25, 1.),
    );
    let reverse_quad_ray = Ray::new(
        quad_point + Vec3::new(-2., -1., 4.),
        Vec3::new(0.5, 0.25, -1.),
    );
    let cuboid_ray = Ray::new(Point3::new(5., 1., -3.), Vec3::new(-0.5, -0.25, 1.));
    compare_cases(
        &heterogeneous_reference,
        &heterogeneous_bvh,
        vec![
            sphere_cases(
                Point3::new(-7., -2., 4.),
                0.8,
                "heterogeneous fixture sphere hit",
            )
            .remove(0),
            RayCase {
                label: "quad hit",
                ray: quad_ray,
                range: 4. ..=4.,
                expected_hit: true,
            },
            RayCase {
                label: "quad hit from reverse side",
                ray: reverse_quad_ray,
                range: 0. ..=10.,
                expected_hit: true,
            },
            RayCase {
                label: "cuboid hit",
                ray: cuboid_ray,
                range: 0. ..=10.,
                expected_hit: true,
            },
            RayCase {
                label: "heterogeneous fixture miss",
                ray: Ray::new(Point3::new(100., 70., -40.), Vec3::new(-1., 0.3, 0.2)),
                range: 0. ..=10.,
                expected_hit: false,
            },
        ],
    );
}

#[test]
fn nearest_hit_is_selected_when_both_root_children_hit() {
    let specs = six_plus_spheres();
    let bvh = BoundedVolumeHierarchy::from(build_list(&specs));
    let BoundedVolumeHierarchy::Node { left, right, .. } = &bvh else {
        panic!("six-plus fixture should take the BVH node-building path");
    };
    let ray = Ray::new(Point3::new(0.15, 0.1, -8.), Vec3::new(0.01, 0.02, 1.));
    let range = 0. ..=100.;
    let left_hit = left.hit(&ray, range.clone()).expect("left child hit");
    let right_hit = right.hit(&ray, range.clone()).expect("right child hit");
    assert_ne!(left_hit.get_t(), right_hit.get_t());
    let expected_nearest = left_hit.get_t().min(right_hit.get_t());
    let tree_hit = bvh.hit(&ray, range).expect("BVH hit");
    assert_close(tree_hit.get_t(), expected_nearest);

    let reference = build_list(&specs);
    let reference_hit = reference.hit(&ray, 0. ..=100.).expect("reference hit");
    assert_hit_records_close(&tree_hit, &reference_hit);
}

#[test]
fn splitting_and_bvh_conversion_conserve_objects_and_bounds() {
    assert_fixture_bounds_and_conservation(&six_plus_spheres());
    assert_fixture_bounds_and_conservation(&heterogeneous_fixture());
}
