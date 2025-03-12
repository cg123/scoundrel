use scoundrel_geometry::*;

use super::octant::octant_transform;
use super::opacity::Opacity;
use super::slope::Slope;
use super::tile_shape::{
    AdamMilazzoTileShape, DiamondTileShape, SquareTileShape, TileShape,
};
use crate::graph::LabeledSpatialGraph;

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
