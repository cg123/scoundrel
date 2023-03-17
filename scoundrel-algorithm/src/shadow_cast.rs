use super::graph::LabeledSpatialGraph;
use scoundrel_geometry::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Opacity {
    Opaque,
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
