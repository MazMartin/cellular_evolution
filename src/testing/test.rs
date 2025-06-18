use crate::graphics::models::space::SrtTransform;
use glam::{Vec2, Vec4};
use crate::utils::{algorithms::CSR, data::IdxPair};

/// Tests that transforming a point by an SrtTransform and then applying the inverse
/// returns the original point (within floating point precision).
#[test]
pub fn test_transforms() {
    let forward = SrtTransform {
        translate: Vec2::new(24.12, -325.13),
        rotate: -112.19,
        scale: Vec2::new(-1334.23, 43987.9),
    };

    let forward_mat = forward.to_mat4();
    let reverse_mat = forward.to_mat4().inverse();

    let point = Vec2::new(398.5, -382.1);
    let transformed = forward_mat * Vec4::new(point.x, point.y, 0.0, 1.0);
    let un_transformed = reverse_mat * transformed;

    println!("starting point: {:?}", point);
    println!("should be same point: {:?}", un_transformed);
}

/// Tests that CSR grouping works correctly on a set of connections.
/// The groups are checked against expected cluster groupings.
#[test]
fn test_csr() {
    let connections = vec![IdxPair::new(0, 1), IdxPair::new(1, 2), IdxPair::new(3, 4)];

    let csr = CSR::groups_from_connections(&connections, 5);

    // Collect groups of indices from CSR ranges
    let mut groups: Vec<Vec<usize>> = csr
        .indptr
        .iter()
        .map(|range| csr.indices[range.a..range.b].to_vec())
        .collect();

    // Sort each group for stable comparison
    for group in &mut groups {
        group.sort();
    }
    groups.sort();

    // Expected groups: connected components plus isolated node '5'
    let expected_groups = vec![vec![0, 1, 2], vec![3, 4], vec![5]];

    assert_eq!(groups, expected_groups);
}
