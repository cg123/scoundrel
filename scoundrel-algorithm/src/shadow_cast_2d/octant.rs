use scoundrel_geometry::*;

/// Returns a transformation matrix for the given octant.
///
/// # Arguments
///
/// * `octant` - The octant number (0-7).
///
/// # Returns
///
/// A `Mat2<i32>` transformation matrix that maps points in octant 0 to the given octant.
pub fn octant_transform(octant: u32) -> Mat2<i32> {
    match octant {
        0 => Mat2::ident(),
        1 => Mat2::row_major(0, 1, 1, 0),
        2 => Mat2::row_major(0, -1, 1, 0),
        3 => Mat2::row_major(-1, 0, 0, 1),
        4 => Mat2::row_major(-1, 0, 0, -1),
        5 => Mat2::row_major(0, -1, -1, 0),
        6 => Mat2::row_major(0, 1, -1, 0),
        7 => Mat2::row_major(1, 0, 0, -1),
        _ => panic!("Invalid octant number: {}", octant),
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_octant_transform() {
        // Test that octant 0 is identity
        let transform = octant_transform(0);
        assert_eq!(transform * Point::new(1, 0), Point::new(1, 0));
        assert_eq!(transform * Point::new(0, 1), Point::new(0, 1));

        // Test that octant 1 rotates 45 degrees
        let transform = octant_transform(1);
        assert_eq!(transform * Point::new(1, 0), Point::new(0, 1));
        assert_eq!(transform * Point::new(0, 1), Point::new(1, 0));

        // Test all octants with a test point
        let test_point = Point::new(3, 1);
        let transformed_points = [
            octant_transform(0) * test_point, // 0°
            octant_transform(1) * test_point, // 45°
            octant_transform(2) * test_point, // 90°
            octant_transform(3) * test_point, // 135°
            octant_transform(4) * test_point, // 180°
            octant_transform(5) * test_point, // 225°
            octant_transform(6) * test_point, // 270°
            octant_transform(7) * test_point, // 315°
        ];

        // Ensure all transformed points are unique
        let unique_points: HashSet<Point> = transformed_points.iter().cloned().collect();
        assert_eq!(
            unique_points.len(),
            8,
            "All octant transformations should produce unique points"
        );
    }

    #[test]
    #[should_panic(expected = "Invalid octant number")]
    fn test_invalid_octant() {
        octant_transform(8); // Should panic
    }
}
