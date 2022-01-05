use super::space::{BaseMap, MapOf, Opacity};
use scoundrel_geometry::*;
use std::cmp::Ordering;

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Slope {
    pub rise: i32,
    pub run: i32,
}

impl Slope {
    pub fn new(mut rise: i32, mut run: i32) -> Slope {
        if run < 0 {
            rise *= -1;
            run *= -1;
        }
        Slope { rise, run }
    }
    pub fn one() -> Slope {
        Slope { rise: 1, run: 1 }
    }
    pub fn zero() -> Slope {
        Slope { rise: 0, run: 0 }
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

fn _cast_light<M: MapOf<Opacity> + BaseMap<Coordinate = Point>, F: FnMut(Point)>(
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

pub fn cast_light<M: MapOf<Opacity> + BaseMap<Coordinate = Point>, F: FnMut(Point)>(
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
            Slope::one(),
            Slope::zero(),
            &mut callback,
        );
    }
}
