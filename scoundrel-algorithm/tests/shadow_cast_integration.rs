use std::collections::{HashMap, HashSet};

use scoundrel_algorithm::{Opacity, cast_light_2d};
use scoundrel_geometry::{Grid2D, Point};

fn _compute_fov(map: &Grid2D<Opacity>, origin: Point, radius: i32) -> HashSet<Point> {
    let mut visible = HashSet::new();
    cast_light_2d(map, origin, radius, |point| {
        visible.insert(point);
    });
    visible
}

#[test]
fn test_fov_origin_always_visible() {
    // Create maps with different configurations
    let maps = [
        Grid2D::new(20, 20, Opacity::Transparent), // Empty map
        Grid2D::from_sparse_points(
            20,
            20,
            Opacity::Transparent,
            vec![Point::new(5, 5)],
            Opacity::Opaque,
        ), // Wall at origin
        Grid2D::from_sparse_points(
            20,
            20,
            Opacity::Opaque,
            vec![
                Point::new(6, 5),
                Point::new(5, 6),
                Point::new(4, 5),
                Point::new(5, 4),
            ],
            Opacity::Transparent,
        ), // Walls surrounding origin
    ];

    for map in &maps {
        let origin = Point::new(5, 5);
        let radius = 10;

        let visible = _compute_fov(map, origin, radius);
        // Origin should always be visible
        assert!(visible.contains(&origin), "Origin should always be visible");
    }
}

#[test]
fn test_fov_walls_block_vision() {
    // Create a map with walls forming a corridor
    let walls = vec![
        // Left wall of corridor
        Point::new(3, 0),
        Point::new(3, 1),
        Point::new(3, 2),
        Point::new(3, 3),
        Point::new(3, 4),
        Point::new(3, 5),
        // Right wall of corridor
        Point::new(7, 0),
        Point::new(7, 1),
        Point::new(7, 2),
        Point::new(7, 3),
        Point::new(7, 4),
        Point::new(7, 5),
    ];

    let map =
        Grid2D::from_sparse_points(11, 11, Opacity::Transparent, walls, Opacity::Opaque);
    let origin = Point::new(5, 0); // Looking down the corridor
    let radius = 10;

    let visible = _compute_fov(&map, origin, radius);

    // Points in the corridor should be visible
    for y in 0..6 {
        for x in 4..7 {
            assert!(
                visible.contains(&Point::new(x, y)),
                "Point in corridor ({}, {}) should be visible",
                x,
                y
            );
        }
    }

    // Points behind walls should not be visible
    let behind_walls = [
        Point::new(2, 3), // Behind left wall
        Point::new(8, 3), // Behind right wall
    ];

    for point in &behind_walls {
        assert!(
            !visible.contains(point),
            "Point behind wall {:?} should not be visible",
            point
        );
    }
}

#[test]
fn test_fov_radius_limits() {
    // Empty map
    let map = Grid2D::new(20, 20, Opacity::Transparent);
    let origin = Point::new(10, 10);

    // Test different radii
    let radii = [1, 3, 5, 8];

    for &radius in &radii {
        let visible = _compute_fov(&map, origin, radius);

        // Check that no points outside the radius are visible
        for point in &visible {
            let dx = point.x - origin.x;
            let dy = point.y - origin.y;
            let distance_squared = dx * dx + dy * dy;

            assert!(
                distance_squared <= radius * radius,
                "Point {:?} outside radius {} should not be visible",
                point,
                radius
            );
        }

        // Verify that the number of visible points increases with radius
        if radius > 1 {
            let smaller_radius = radius - 1;
            let visible_smaller = _compute_fov(&map, origin, smaller_radius);

            assert!(
                visible.len() > visible_smaller.len(),
                "FOV with radius {} should show more points than radius {}",
                radius,
                smaller_radius
            );
        }
    }
}

#[test]
fn test_fov_complex_scenario() {
    // Create a more complex map with rooms and doorways
    let walls = vec![
        // Room 1 walls
        Point::new(2, 2),
        Point::new(3, 2),
        Point::new(4, 2),
        Point::new(5, 2),
        Point::new(6, 2),
        Point::new(7, 2),
        Point::new(2, 3),
        Point::new(7, 3),
        Point::new(2, 4),
        // doorway at (7, 4)
        Point::new(2, 5),
        Point::new(7, 5),
        Point::new(2, 6),
        Point::new(3, 6),
        Point::new(4, 6),
        Point::new(5, 6),
        Point::new(6, 6),
        Point::new(7, 6),
        // Room 2 walls
        Point::new(9, 2),
        Point::new(10, 2),
        Point::new(11, 2),
        Point::new(12, 2),
        Point::new(13, 2),
        Point::new(14, 2),
        Point::new(9, 3),
        Point::new(14, 3),
        // doorway at (9, 4)
        Point::new(14, 4),
        Point::new(9, 5),
        Point::new(14, 5),
        Point::new(9, 6),
        Point::new(10, 6),
        Point::new(11, 6),
        Point::new(12, 6),
        Point::new(13, 6),
        Point::new(14, 6),
        // pillar in center of room 2
        Point::new(11, 4),
    ];

    let map =
        Grid2D::from_sparse_points(18, 10, Opacity::Transparent, walls, Opacity::Opaque);

    // Test visibility from room 1
    let origin_room1 = Point::new(4, 4);
    let visible_room1 = _compute_fov(&map, origin_room1, 10);

    // Points in room 1 should be visible
    let room1_points = [
        Point::new(3, 3),
        Point::new(6, 5),
        Point::new(3, 5),
        Point::new(6, 3),
    ];
    for point in &room1_points {
        assert!(
            visible_room1.contains(point),
            "Point in room 1 {:?} should be visible",
            point
        );
    }

    // Some points in room 2 should be visible through doorway
    let visible_through_door = [
        Point::new(9, 4), // Center of doorway
        Point::new(10, 4),
        Point::new(11, 4), // Pillar in room 2
    ];
    for point in &visible_through_door {
        assert!(
            visible_room1.contains(point),
            "Point visible through doorway {:?} should be visible",
            point
        );
    }

    // Center of back wall of room 2 should not be visible
    assert!(
        !visible_room1.contains(&Point::new(14, 4)),
        "Point behind wall in room 2 should not be visible"
    );
}

#[test]
fn test_fov_symmetric_property() {
    // FOV should have a roughly symmetric property - if A can see B, B can see A
    // (with some precision differences due to algorithm implementation)

    // Empty map to test true symmetry
    let map = Grid2D::new(15, 15, Opacity::Transparent);
    let radius = 8;

    // Create a visibility map for each position
    let mut can_see_from = HashMap::new();

    // Sample points for efficiency
    let sample_points = [
        Point::new(3, 3),
        Point::new(7, 7),
        Point::new(10, 3),
        Point::new(3, 10),
        Point::new(10, 10),
    ];

    // Calculate FOV for each sample point
    for &origin in &sample_points {
        can_see_from.insert(origin, _compute_fov(&map, origin, radius));
    }

    // Test symmetry
    for &point_a in &sample_points {
        for &point_b in &sample_points {
            if point_a == point_b {
                continue;
            }

            let a_sees_b = can_see_from.get(&point_a).unwrap().contains(&point_b);
            let b_sees_a = can_see_from.get(&point_b).unwrap().contains(&point_a);

            assert_eq!(
                a_sees_b, b_sees_a,
                "FOV symmetry broken: visibility from {:?} to {:?} is {}, but visibility from {:?} to {:?} is {}",
                point_a, point_b, a_sees_b, point_b, point_a, b_sees_a
            );
        }
    }
}
