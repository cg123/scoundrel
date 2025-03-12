use scoundrel_geometry::*;

use super::opacity::Opacity;
use super::slope::Slope;
use crate::graph::LabeledSpatialGraph;

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
