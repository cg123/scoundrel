mod algorithm;
mod octant;
mod opacity;
mod slope;
mod tile_shape;

pub use algorithm::{cast_light_2d, cast_light_2d_beveled, cast_light_2d_diamond};
pub use opacity::Opacity;
pub use slope::Slope;
pub use tile_shape::{DiamondTileShape, SquareTileShape, TileShape};

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use scoundrel_geometry::*;

    use super::*;

    fn _compute_fov(map: &Grid2D<Opacity>, origin: Point, radius: i32) -> HashSet<Point> {
        let mut visible = HashSet::new();
        cast_light_2d(map, origin, radius, |point| {
            visible.insert(point);
        });
        visible
    }

    #[test]
    fn test_cast_light_basic() {
        // Empty 10x10 map
        let map = Grid2D::new(10, 10, Opacity::Transparent);
        let origin = Point::new(5, 5);
        let range = 3;

        // Collect visible points
        let visible_points = _compute_fov(&map, origin, range);
        // Origin should be visible
        assert!(visible_points.contains(&origin), "Origin should be visible");

        // All points within range should be visible in an empty map
        for x in origin.x - range..=origin.x + range {
            for y in origin.y - range..=origin.y + range {
                let p = Point::new(x, y);
                let dist_squared = (p.x - origin.x).pow(2) + (p.y - origin.y).pow(2);
                if dist_squared <= range * range {
                    assert!(
                        visible_points.contains(&p),
                        "Point {:?} should be visible",
                        p
                    );
                }
            }
        }
    }

    #[test]
    fn test_cast_light_with_wall() {
        // 10x10 map with a wall that blocks visibility
        let walls = vec![
            Point::new(5, 7), // Wall above origin
        ];
        let map = Grid2D::from_sparse_points(
            10,
            10,
            Opacity::Transparent,
            walls,
            Opacity::Opaque,
        );
        let origin = Point::new(5, 5);
        let range = 5;

        // Collect visible points
        let visible_points = _compute_fov(&map, origin, range);

        // Points directly behind the wall should not be visible
        let shadow_points = [Point::new(5, 8), Point::new(5, 9)];

        for point in &shadow_points {
            assert!(
                !visible_points.contains(point),
                "Point {:?} should be in shadow",
                point
            );
        }

        // Points that aren't behind the wall should still be visible
        let visible = [Point::new(4, 8), Point::new(6, 8)];

        for point in &visible {
            assert!(
                visible_points.contains(point),
                "Point {:?} should be visible",
                point
            );
        }
    }

    #[test]
    fn test_different_algorithms() {
        // Test that different algorithms produce different results
        let walls = vec![
            Point::new(6, 0),
            Point::new(6, 1),
            Point::new(6, 2),
            Point::new(6, 3),
            Point::new(6, 4),
            Point::new(2, 1),
            Point::new(4, 3),
        ];
        let map = Grid2D::from_sparse_points(
            8,
            5,
            Opacity::Transparent,
            walls,
            Opacity::Opaque,
        );
        let origin = Point::new(0, 1);
        let range = 100;

        // Collect visible points with different algorithms
        let mut square_visible = HashSet::new();
        cast_light_2d(&map, origin, range, |point| {
            square_visible.insert(point);
        });

        let mut diamond_visible = HashSet::new();
        cast_light_2d_diamond(&map, origin, range, |point| {
            diamond_visible.insert(point);
        });

        let mut beveled_visible = HashSet::new();
        cast_light_2d_beveled(&map, origin, range, |point| {
            beveled_visible.insert(point);
        });

        println!(
            "Square map:\n{}",
            draw_visible_points(&map, &square_visible, &origin)
        );
        println!(
            "Diamond map:\n{}",
            draw_visible_points(&map, &diamond_visible, &origin)
        );
        println!(
            "Beveled map (Adam Milazzo):\n{}",
            draw_visible_points(&map, &beveled_visible, &origin)
        );

        // The algorithms should produce different results
        assert_ne!(
            square_visible, diamond_visible,
            "Square and diamond algorithms should produce different results"
        );
        assert_ne!(
            square_visible, beveled_visible,
            "Square and beveled algorithms should produce different results"
        );
        assert_ne!(
            diamond_visible, beveled_visible,
            "Diamond and beveled algorithms should produce different results"
        );
    }

    fn draw_visible_points(
        map: &Grid2D<Opacity>,
        visible: &HashSet<Point>,
        origin: &Point,
    ) -> String {
        let mut visible_map = String::new();
        for y in 0..map.height() {
            for x in 0..map.width() {
                let p = Point::new(x, map.height() - y - 1);
                let is_opaque = map.get(p) == Some(&Opacity::Opaque);
                let is_visible = visible.contains(&p);
                let c = if p == *origin {
                    '@'
                } else if is_opaque && is_visible {
                    '#'
                } else if is_opaque && !is_visible {
                    'X'
                } else if is_visible && !is_opaque {
                    '.'
                } else {
                    ' '
                };
                visible_map.push(c);
            }
            visible_map.push('\n');
        }
        visible_map
    }

    #[test]
    fn test_diagonal_wall() {
        // 6x6 map with a diagonal wall
        let walls = vec![
            Point::new(0, 0),
            Point::new(1, 1),
            Point::new(2, 2),
            Point::new(3, 3),
            Point::new(4, 4),
            Point::new(5, 5),
        ];
        let map = Grid2D::from_sparse_points(
            6,
            6,
            Opacity::Transparent,
            walls,
            Opacity::Opaque,
        );
        let origin = Point::new(0, 5);
        let opposite_corner = Point::new(5, 0);
        let test_point = Point::new(5, 2);
        let range = 10;

        let mut square_visible = HashSet::new();
        cast_light_2d(&map, origin, range, |point| {
            square_visible.insert(point);
        });

        let mut diamond_visible = HashSet::new();
        cast_light_2d_diamond(&map, origin, range, |point| {
            diamond_visible.insert(point);
        });

        let mut beveled_visible = HashSet::new();
        cast_light_2d_beveled(&map, origin, range, |point| {
            beveled_visible.insert(point);
        });

        println!(
            "Square map:\n{}",
            draw_visible_points(&map, &square_visible, &origin)
        );
        println!(
            "Diamond map:\n{}",
            draw_visible_points(&map, &diamond_visible, &origin)
        );
        println!(
            "Beveled map (Adam Milazzo):\n{}",
            draw_visible_points(&map, &beveled_visible, &origin)
        );

        assert!(
            !square_visible.contains(&test_point),
            "Square algorithm should not see test point"
        );
        assert!(
            diamond_visible.contains(&opposite_corner)
                && diamond_visible.contains(&test_point),
            "Diamond algorithm should see opposite corner and test point"
        );
        assert!(
            beveled_visible.contains(&opposite_corner)
                && !beveled_visible.contains(&test_point),
            "Beveled algorithm should see opposite corner but not test point"
        );
    }
}
