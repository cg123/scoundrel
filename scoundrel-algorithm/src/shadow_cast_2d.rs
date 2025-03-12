use std::cmp::Ordering;

use scoundrel_geometry::*;

use super::graph::LabeledSpatialGraph;

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
pub struct Slope {
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

/// Trait that defines how tile shapes are interpreted for FOV calculations.
///
/// By implementing this trait differently, we can create different FOV algorithms
/// that represent tiles with different shapes (square, diamond, beveled corners, etc).
pub trait TileShape {
    /// Calculate the high slope value for a tile at coordinates (x, y)
    fn tile_slope_high(&self, x: i32, y: i32) -> Slope;

    /// Calculate the low slope value for a tile at coordinates (x, y)
    fn tile_slope_low(&self, x: i32, y: i32) -> Slope;

    /// Calculate the previous tile's low slope for transitions
    fn prev_tile_slope_low(&self, x: i32, y: i32) -> Slope;
}

/// Standard square tile shape used in basic shadowcasting
pub struct SquareTileShape;

impl TileShape for SquareTileShape {
    fn tile_slope_high(&self, x: i32, y: i32) -> Slope {
        Slope::new(2 * y + 1, 2 * x - 1)
    }

    fn tile_slope_low(&self, x: i32, y: i32) -> Slope {
        Slope::new(2 * y - 1, 2 * x + 1)
    }

    fn prev_tile_slope_low(&self, x: i32, y: i32) -> Slope {
        Slope::new(2 * y + 1, 2 * x + 1)
    }
}

/// Diamond-shaped tiles for smoother FOV
pub struct DiamondTileShape;

impl TileShape for DiamondTileShape {
    fn tile_slope_high(&self, x: i32, y: i32) -> Slope {
        Slope::new(y * 2 + 1, x * 2)
    }

    fn tile_slope_low(&self, x: i32, y: i32) -> Slope {
        Slope::new(y * 2 - 1, x * 2)
    }

    fn prev_tile_slope_low(&self, x: i32, y: i32) -> Slope {
        Slope::new(y * 2 + 1, x * 2)
    }
}

/// Implementation of the tile shape used in Adam Milazzo's algorithm.
///
/// The algorithm is described in detail in his blog post:
/// http://www.adammil.net/blog/v125_Roguelike_Vision_Algorithms.html
pub struct AdamMilazzoTileShape<'a, M: LabeledSpatialGraph<Opacity, NodeHandle = Point>> {
    map: &'a M,
    origin: Point,
    transform: Mat2<i32>,
}

impl<'a, M: LabeledSpatialGraph<Opacity, NodeHandle = Point>>
    AdamMilazzoTileShape<'a, M>
{
    pub fn new(map: &'a M, origin: Point, transform: Mat2<i32>) -> Self {
        Self {
            map,
            origin,
            transform,
        }
    }

    /// Maps a point from octant 0 coordinates to world coordinates and checks if it blocks light
    fn blocks_light(&self, x: i32, y: i32) -> bool {
        let map_pt = self.origin + self.transform * Point::new(y, x);
        self.map.get(map_pt) == Some(Opacity::Opaque)
    }
}

impl<'a, M: LabeledSpatialGraph<Opacity, NodeHandle = Point>> TileShape
    for AdamMilazzoTileShape<'a, M>
{
    /*
     * Diagram of tile parts from Adam Milazzo's code:
     *    g         center:        y / x
     * a------b   a top left:      (y*2+1) / (x*2-1)   i inner top left:      (y*4+1) / (x*4-1)
     * |  /\  |   b top right:     (y*2+1) / (x*2+1)   j inner top right:     (y*4+1) / (x*4+1)
     * |i/__\j|   c bottom left:   (y*2-1) / (x*2-1)   k inner bottom left:   (y*4-1) / (x*4-1)
     *e|/|  |\|f  d bottom right:  (y*2-1) / (x*2+1)   m inner bottom right:  (y*4-1) / (x*4+1)
     * |\|__|/|   e middle left:   (y*2) / (x*2-1)
     * |k\  /m|   f middle right:  (y*2) / (x*2+1)     a-d are the corners of the tile
     * |  \/  |   g top center:    (y*2+1) / (x*2)     e-h are the corners of the inner (wall) diamond
     * c------d   h bottom center: (y*2-1) / (x*2)     i-m are the corners of the inner square (1/2 tile width)
     *    h
     */
    fn tile_slope_high(&self, x: i32, y: i32) -> Slope {
        /*
         * In Milazzo's algorithm, this is for the tile_slope_high which affects
         * transitions from floor to wall. Referred to as "upper" or "top" in the comments.
         */
        if self.blocks_light(x, y) {
            // If this is a wall tile, determine if its top-left corner is beveled.
            // The corner is beveled if the tiles above and to the left are clear.

            // We know the current tile is a wall, so we need to check if the tile above is clear
            if !self.blocks_light(x, y + 1) {
                // Beveled corner - use top center (g in diagram)
                return Slope::new(2 * y + 1, 2 * x);
            } else {
                // Non-beveled corner - use top left (a in diagram)
                return Slope::new(2 * y + 1, 2 * x - 1);
            }
        } else {
            // For floor tiles, just use the top-left corner (a in diagram)
            return Slope::new(2 * y + 1, 2 * x - 1);
        }
    }

    fn tile_slope_low(&self, x: i32, y: i32) -> Slope {
        /*
         * In Milazzo's algorithm, this is for the tile_slope_low which affects
         * transitions when we check if a tile is in shadow. Referred to as "lower"
         * or "bottom" in the comments.
         */
        if self.blocks_light(x, y) {
            // If we're in a wall tile, we need to check if the bottom-right corner is beveled.
            // The corner is beveled if the tiles below and to the right are clear.

            // We know current tile is a wall, we can check if the tile to the right is clear
            if !self.blocks_light(x + 1, y) {
                // Beveled corner - use bottom center (h in diagram)
                return Slope::new(2 * y - 1, 2 * x);
            } else {
                // Non-beveled corner - use bottom right (d in diagram)
                return Slope::new(2 * y - 1, 2 * x + 1);
            }
        } else {
            // For floor tiles, use the bottom-right corner (d in diagram)
            return Slope::new(2 * y - 1, 2 * x + 1);
        }
    }

    fn prev_tile_slope_low(&self, x: i32, y: i32) -> Slope {
        /*
         * This is used when we find a transition from wall to floor, and we need
         * to adjust the top vector (slope_high).
         *
         * From Adam's code: "if we found a transition from opaque to clear, adjust the top vector downwards"
         */

        // Check if the opaque tile has a beveled bottom-right corner
        // The corner is beveled if the tiles below and to the right are clear
        // We know the tile at (x,y) was a wall and we're now in a clear tile, so check to the right
        if !self.blocks_light(x + 1, y) {
            // Beveled - use bottom center (h in diagram)
            return Slope::new(2 * y, 2 * x);
        } else {
            // Not beveled - use bottom right (d in diagram)
            return Slope::new(2 * y, 2 * x + 1);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn _cast_light<M, F, T>(
    map: &M,
    origin: Point,
    range: i32,
    transform: Mat2<i32>,
    x: i32,
    mut slope_high: Slope,
    slope_low: Slope,
    tile_shape: &T,
    callback: &mut F,
) where
    M: LabeledSpatialGraph<Opacity, NodeHandle = Point>,
    F: FnMut(Point),
    T: TileShape,
{
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
        let tile_slope_high = tile_shape.tile_slope_high(x, y);
        let tile_slope_low = tile_shape.tile_slope_low(x, y);

        if tile_slope_low > slope_high {
            continue;
        }
        if tile_slope_high < slope_low {
            break;
        }

        let in_range = x * x + y * y <= range * range;
        let map_pt = origin + transform * Point::new(y, x);
        let opaque = map.get(map_pt) != Some(Opacity::Transparent);
        if in_range {
            callback(map_pt);
        }

        if prev_opaque && !opaque {
            slope_high = tile_shape.prev_tile_slope_low(x, y);
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
                tile_shape,
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
            tile_shape,
            callback,
        );
    }
}

/// Casts light in all directions from the given origin point using square tiles.
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
pub fn cast_light_2d<
    M: LabeledSpatialGraph<Opacity, NodeHandle = Point>,
    F: FnMut(Point),
>(
    map: &M,
    origin: Point,
    range: i32,
    mut callback: F,
) {
    callback(origin);
    let tile_shape = SquareTileShape;
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
            &tile_shape,
            &mut callback,
        );
    }
}

/// Casts light in all directions from the given origin point using diamond-shaped tiles.
///
/// This variant produces a smoother field of view that's useful for many roguelike games.
///
/// # Arguments
///
/// * `map` - The map to cast light on.
/// * `origin` - The origin point to cast light from.
/// * `range` - The maximum range of the light.
/// * `callback` - A callback function to call for each lit tile.
pub fn cast_light_2d_diamond<
    M: LabeledSpatialGraph<Opacity, NodeHandle = Point>,
    F: FnMut(Point),
>(
    map: &M,
    origin: Point,
    range: i32,
    mut callback: F,
) {
    callback(origin);
    let tile_shape = DiamondTileShape;
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
            &tile_shape,
            &mut callback,
        );
    }
}

/// Casts light in all directions from the given origin point using beveled corner tiles.
///
/// This is an implementation of Adam Milazzo's algorithm which considers how
/// adjacent walls affect visibility. It creates more natural-looking shadows around corners.
///
/// # Arguments
///
/// * `map` - The map to cast light on.
/// * `origin` - The origin point to cast light from.
/// * `range` - The maximum range of the light.
/// * `callback` - A callback function to call for each lit tile.
pub fn cast_light_2d_beveled<
    M: LabeledSpatialGraph<Opacity, NodeHandle = Point>,
    F: FnMut(Point),
>(
    map: &M,
    origin: Point,
    range: i32,
    mut callback: F,
) {
    callback(origin);
    for octant in 0..8 {
        let transform = octant_transform(octant);
        let tile_shape = AdamMilazzoTileShape::new(map, origin, transform);
        _cast_light(
            map,
            origin,
            range,
            transform,
            1,
            Slope::ONE,
            Slope::ZERO,
            &tile_shape,
            &mut callback,
        );
    }
}

#[cfg(test)]
mod tests {
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
