use super::graph::LabeledSpatialGraph;
use scoundrel_geometry::*;
use std::cmp::Ordering;

/// Represents whether a tile or object blocks light for field of view calculations.
///
/// This enum is used by the shadowcasting algorithm to determine if light
/// can pass through a tile when calculating field of view.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Opacity {
    /// The tile blocks light completely (walls, solid objects).
    Opaque,
    /// The tile allows light to pass through (empty space, windows).
    Transparent,
}

/// Returns a transformation matrix for the given octant.
///
/// # Arguments
///
/// * `octant` - The octant number (0-7).
///
/// # Returns
///
/// A `Mat2<i32>` transformation matrix that maps points in octant 0 to the given octant.
fn octant_transform(octant: u32) -> Mat2<i32> {
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

/// Represents a slope as a rise and run.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Slope {
    pub rise: i32,
    pub run: i32,
}

impl Slope {
    pub const ONE: Slope = Slope::new(1, 1);
    pub const ZERO: Slope = Slope::new(0, 0);

    pub const fn new(mut rise: i32, mut run: i32) -> Slope {
        if run < 0 {
            rise *= -1;
            run *= -1;
        }
        Slope { rise, run }
    }
}

impl PartialOrd<Self> for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Self) -> Ordering {
        // sy / sx <>= oy / ox
        // sy * ox / sx <>= oy
        // sy * ox <>= oy * sx
        // *given* our invariant that sx, ox > 0
        (self.rise * other.run).cmp(&(other.rise * self.run))
    }
}

#[allow(clippy::too_many_arguments)]
fn _cast_light<M: LabeledSpatialGraph<Opacity, NodeHandle = Point>, F: FnMut(Point)>(
    map: &M,
    origin: Point,
    range: i32,
    transform: Mat2<i32>,
    x: i32,
    mut slope_high: Slope,
    slope_low: Slope,
    callback: &mut F,
) {
    if slope_high < slope_low || x > range {
        return;
    }

    let y0 = if slope_low.run > 0 {
        ((2 * x - 1) * slope_low.rise - slope_low.run) / (2 * slope_low.run)
    } else {
        0
    };

    let mut prev_opaque = false;
    for y in (y0..=x).rev() {
        let tile_slope_high = Slope::new(2 * y + 1, 2 * x - 1);
        let tile_slope_low = Slope::new(2 * y - 1, 2 * x + 1);
        let prev_tile_slope_low = Slope::new(2 * y + 1, 2 * x + 1);
        if tile_slope_low > slope_high {
            continue;
        }
        if tile_slope_high < slope_low {
            break;
        }

        let in_range = y * y + x * x <= range * range;
        let map_pt = origin + transform * Point::new(y, x);
        let opaque = map.get(map_pt) != Some(Opacity::Transparent);
        if in_range {
            callback(map_pt);
        }

        if prev_opaque && !opaque {
            slope_high = prev_tile_slope_low;
        }
        if !prev_opaque && opaque {
            _cast_light(
                map,
                origin,
                range,
                transform,
                x + 1,
                slope_high,
                tile_slope_high,
                callback,
            );
        }
        prev_opaque = opaque;
    }
    if !prev_opaque {
        _cast_light(
            map,
            origin,
            range,
            transform,
            x + 1,
            slope_high,
            slope_low,
            callback,
        );
    }
}

/// Casts light in all directions from the given origin point.
///
/// # Arguments
///
/// * `map` - The map to cast light on.
/// * `origin` - The origin point to cast light from.
/// * `range` - The maximum range of the light.
/// * `callback` - A callback function to call for each lit tile.
///
/// # Type Parameters
///
/// * `M` - The type of the map.
/// * `F` - The type of the callback function.
pub fn cast_light<M: LabeledSpatialGraph<Opacity, NodeHandle = Point>, F: FnMut(Point)>(
    map: &M,
    origin: Point,
    range: i32,
    mut callback: F,
) {
    callback(origin);
    for octant in 0..8 {
        let transform = octant_transform(octant);
        _cast_light(
            map,
            origin,
            range,
            transform,
            1,
            Slope::ONE,
            Slope::ZERO,
            &mut callback,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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

    #[test]
    fn test_slope_comparison() {
        // Test slope ordering
        let s1 = Slope::new(1, 2); // 0.5
        let s2 = Slope::new(1, 1); // 1.0
        let s3 = Slope::new(2, 1); // 2.0
        let s4 = Slope::new(-1, -2); // 0.5 (normalized to 1/2)

        assert!(s1 < s2);
        assert!(s2 < s3);
        assert_eq!(s1, s4); // Same slope after normalization

        // Test negative run normalization
        let s5 = Slope::new(1, -2);
        let s6 = Slope::new(-1, 2);
        assert_eq!(s5, s6);
    }

    #[test]
    fn test_slope_constants() {
        assert_eq!(Slope::ZERO, Slope::new(0, 0));
        assert_eq!(Slope::ONE, Slope::new(1, 1));
    }

    fn _compute_fov(map: &Grid2D<Opacity>, origin: Point, radius: i32) -> HashSet<Point> {
        let mut visible = HashSet::new();
        cast_light(map, origin, radius, |point| {
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
        let map = Grid2D::from_sparse_points(10, 10, Opacity::Transparent, walls, Opacity::Opaque);
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
}
