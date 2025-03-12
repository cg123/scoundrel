use std::ops;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tui")]
use tui::layout::{Rect, Size};

use crate::Vector2;

/// A bounding box in two-dimensional space defined by a minimum and maximum point.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Bounds<T: Copy> {
    /// The minimum point of the bounding box (inclusive).
    pub min: Vector2<T>,
    /// The maximum point of the bounding box (exclusive).
    pub max: Vector2<T>,
}

impl<T: Copy> Bounds<T> {
    /// Creates a new bounding box with the specified minimum and maximum points.
    pub fn with_points(min: Vector2<T>, max: Vector2<T>) -> Bounds<T> {
        Bounds { min, max }
    }
}

impl<T: Copy + PartialOrd> Bounds<T> {
    /// Returns true if the specified point is contained within the bounding box, false otherwise.
    pub fn contains(&self, point: Vector2<T>) -> bool {
        point.x >= self.min.x
            && point.x < self.max.x
            && point.y >= self.min.y
            && point.y < self.max.y
    }

    /// Returns true if the bounding box intersects with the specified other bounding box, false otherwise.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

impl Bounds<i32> {
    /// Calls the specified closure `f` for each integer point within the bounding box.
    pub fn for_each<F: FnMut(Vector2<i32>)>(&self, mut f: F) {
        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                f(Vector2::new(x, y));
            }
        }
    }

    /// Returns a `Vec` containing all integer points within the bounding box.
    pub fn contained_points(&self) -> Vec<Vector2<i32>> {
        let mut res = vec![];
        self.for_each(|pt| res.push(pt));
        res
    }

    /// Calls the specified closure `f` for each integer point along the border of the bounding box.
    pub fn for_each_border<F: FnMut(Vector2<i32>)>(&self, mut f: F) {
        for x in self.min.x..self.max.x {
            f(Vector2::new(x, self.min.y));
            f(Vector2::new(x, self.max.y - 1));
        }
        for y in (self.min.y + 1)..(self.max.y - 1) {
            f(Vector2::new(self.min.x, y));
            f(Vector2::new(self.max.x - 1, y));
        }
    }
}

impl<T: Copy + ops::Sub<Output = Tp>, Tp: Copy> Bounds<T> {
    /// Returns a `Vector2` representing the size of this bounding box.
    pub fn size(&self) -> Vector2<Tp> {
        self.max - self.min
    }
}

impl<T: Copy + ops::Add<Output = T>> Bounds<T> {
    /// Returns a new `Bounds` instance with the specified minimum point and size.
    pub fn with_size(min: Vector2<T>, size: Vector2<T>) -> Bounds<T> {
        Bounds {
            min,
            max: min + size,
        }
    }
}

impl<T: Copy + ops::Add<T, Output = Tp>, Tp: Copy> ops::Add<Vector2<T>> for Bounds<T> {
    type Output = Bounds<Tp>;

    fn add(self, rhs: Vector2<T>) -> Self::Output {
        Bounds {
            min: self.min + rhs,
            max: self.max + rhs,
        }
    }
}

impl<T: Copy + ops::Sub<Output = Tp>, Tp: Copy> ops::Sub<Vector2<T>> for Bounds<T> {
    type Output = Bounds<Tp>;

    fn sub(self, rhs: Vector2<T>) -> Self::Output {
        Bounds {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

#[cfg(feature = "tui")]
impl<T: Copy + From<u16>> From<tui::layout::Rect> for Bounds<T> {
    fn from(other: Rect) -> Self {
        let min = Vector2::new(other.x.into(), other.y.into());
        let max = Vector2::new(
            (other.x + other.width).into(),
            (other.y + other.height).into(),
        );
        Bounds { min, max }
    }
}
#[cfg(feature = "tui")]
impl<T: Copy + TryInto<u16> + ops::Sub<T, Output = T>> TryInto<tui::layout::Rect>
    for Bounds<T>
{
    type Error = <T as TryInto<u16>>::Error;

    fn try_into(self) -> Result<Rect, Self::Error> {
        Ok(Rect {
            x: self.min.x.try_into()?,
            y: self.min.y.try_into()?,
            width: self.size().x.try_into()?,
            height: self.size().y.try_into()?,
        })
    }
}

#[cfg(feature = "tui")]
impl<T: Copy + From<u16>> From<Size> for Vector2<T> {
    fn from(other: Size) -> Self {
        Vector2::new(other.width.into(), other.height.into())
    }
}

impl<
        T: Copy
            + ops::Add<T, Output = T>
            + ops::Sub<T, Output = T>
            + ops::Div<T, Output = T>
            + From<i32>,
    > Bounds<T>
{
    /// Returns a new `Bounds` instance with the specified center point and size.
    pub fn with_center(center: Vector2<T>, size: Vector2<T>) -> Bounds<T> {
        let min = center - (size / 2_i32.into());
        let max = min + size;
        Bounds { min, max }
    }

    pub fn center(&self) -> Vector2<T> {
        (self.min + self.max) / 2_i32.into()
    }

    pub fn quadrant(&self, index: usize) -> Self {
        let (x_lt, y_lt) = match index {
            0 => (false, false),
            1 => (true, false),
            2 => (true, true),
            3 => (false, true),
            _ => panic!("invalid quadrant number"),
        };

        let center = self.center();

        let min = Vector2::new(
            if x_lt { self.min.x } else { center.x },
            if y_lt { self.min.y } else { center.y },
        );
        let max = Vector2::new(
            if x_lt { center.x } else { self.max.x },
            if y_lt { center.y } else { self.max.y },
        );
        Self::with_points(min, max)
    }
}

impl<
        T: Copy
            + ops::Add<T, Output = T>
            + ops::Sub<T, Output = T>
            + ops::Div<T, Output = T>
            + From<i32>
            + PartialOrd<T>,
    > Bounds<T>
{
    pub fn containing_quadrant_idx(&self, query: Vector2<T>) -> usize {
        let center = self.center();
        let x_lt = query.x < center.x;
        let y_lt = query.y < center.y;
        match (x_lt, y_lt) {
            (false, false) => 0,
            (true, false) => 1,
            (true, true) => 2,
            (false, true) => 3,
        }
    }
}

impl<T: Copy + Ord + ops::Add<T, Output = T>> Bounds<T> {
    pub fn sub_rect(&self, offset: Vector2<T>, size: Vector2<T>) -> Bounds<T> {
        let pt1 = self.min + offset;
        let pt2 = Vector2::new(
            std::cmp::min(pt1.x + size.x, self.max.x),
            std::cmp::min(pt1.y + size.y, self.max.y),
        );
        Bounds::with_points(pt1, pt2)
    }
}

impl<T: Copy + Ord> Bounds<T> {
    /// Returns the closest point within the bounding box with respect to a given query point.
    pub fn closest_pt(&self, query: Vector2<T>) -> Vector2<T> {
        Vector2::new(
            std::cmp::min(std::cmp::max(self.min.x, query.x), self.max.x),
            std::cmp::min(std::cmp::max(self.min.y, query.y), self.max.y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_creation() {
        let bounds = Bounds::with_points(Vector2::new(2, 3), Vector2::new(5, 7));
        assert_eq!(bounds.min, Vector2::new(2, 3));
        assert_eq!(bounds.max, Vector2::new(5, 7));
    }

    #[test]
    fn test_contains() {
        let b = Bounds::with_points(Vector2::new(0, 0), Vector2::new(10, 10));
        assert!(b.contains(Vector2::new(5, 5)));
        assert!(b.contains(Vector2::new(0, 0)));
        assert!(b.contains(Vector2::new(9, 9)));
        assert!(!b.contains(Vector2::new(10, 10)));
        assert!(!b.contains(Vector2::new(-1, -1)));
    }

    #[test]
    fn test_intersects() {
        let b1 = Bounds::with_points(Vector2::new(0, 0), Vector2::new(10, 10));
        let b2 = Bounds::with_points(Vector2::new(5, 5), Vector2::new(15, 15));
        let b3 = Bounds::with_points(Vector2::new(-5, -5), Vector2::new(5, -1));
        assert!(b1.intersects(&b2));
        assert!(b2.intersects(&b1));
        assert!(b1.intersects(&b1));
        assert!(!b1.intersects(&b3));
        assert!(!b3.intersects(&b1));
        assert!(!b3.intersects(&b2));
        assert!(!b2.intersects(&b3));
    }

    #[test]
    fn test_for_each() {
        let b = Bounds::with_points(Vector2::new(0, 0), Vector2::new(3, 3));
        let mut count = 0;
        b.for_each(|_| count += 1);
        assert_eq!(count, 9);
    }

    #[test]
    fn test_contained_points() {
        let b = Bounds::with_points(Vector2::new(0, 0), Vector2::new(3, 3));
        let points = b.contained_points();
        assert_eq!(points.len(), 9);
        for x in 0..3 {
            for y in 0..3 {
                assert!(points.contains(&Vector2::new(x, y)));
            }
        }
    }

    #[test]
    fn test_for_each_border() {
        let b = Bounds::with_points(Vector2::new(0, 0), Vector2::new(3, 3));
        let mut count = 0;
        b.for_each_border(|_| count += 1);
        assert_eq!(count, 8);
    }

    #[test]
    fn test_size() {
        let b = Bounds::with_points(Vector2::new(0, 0), Vector2::new(10, 20));
        let size = b.size();
        assert_eq!(size.x, 10);
        assert_eq!(size.y, 20);
    }

    #[test]
    fn test_with_size() {
        let b = Bounds::with_size(Vector2::new(0, 0), Vector2::new(10, 20));
        assert_eq!(b.min, Vector2::new(0, 0));
        assert_eq!(b.max, Vector2::new(10, 20));
    }
}
