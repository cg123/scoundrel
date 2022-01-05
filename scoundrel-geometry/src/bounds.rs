use crate::Vector2;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ops;
#[cfg(feature = "tui")]
use tui::layout::Rect;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Bounds<T: Copy> {
    pub min: Vector2<T>,
    pub max: Vector2<T>,
}
impl<T: Copy> Bounds<T> {
    pub fn with_points(min: Vector2<T>, max: Vector2<T>) -> Bounds<T> {
        Bounds { min, max }
    }
}

impl<T: Copy + PartialOrd> Bounds<T> {
    pub fn contains(&self, point: Vector2<T>) -> bool {
        point.x >= self.min.x
            && point.x < self.max.x
            && point.y >= self.min.y
            && point.y < self.max.y
    }
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }
}

impl Bounds<i32> {
    pub fn for_each<F: FnMut(Vector2<i32>)>(&self, mut f: F) {
        for y in self.min.y..self.max.y {
            for x in self.min.x..self.max.x {
                f(Vector2::new(x, y));
            }
        }
    }
    pub fn contained_points(&self) -> Vec<Vector2<i32>> {
        let mut res = vec![];
        self.for_each(|pt| res.push(pt));
        res
    }

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
    pub fn size(&self) -> Vector2<Tp> {
        self.max - self.min
    }
}
impl<T: Copy + ops::Add<Output = T>> Bounds<T> {
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
impl<T: Copy + TryInto<u16> + ops::Sub<T, Output = T>> TryInto<tui::layout::Rect> for Bounds<T> {
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

impl<
        T: Copy
            + ops::Add<T, Output = T>
            + ops::Sub<T, Output = T>
            + ops::Div<T, Output = T>
            + From<i32>,
    > Bounds<T>
{
    pub fn with_center(center: Vector2<T>, size: Vector2<T>) -> Bounds<T> {
        let min = center - (size / 2_i32.into());
        let max = min + size;
        Bounds { min, max }
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

/*impl<T: TryInto<Tp> + Copy, Tp: Copy> TryInto<Bounds<Tp>> for Bounds<T> {
    type Error = <T as TryInto<Tp>>::Error;

    fn try_into(self) -> Result<Bounds<Tp>, Self::Error> {
        Ok(Bounds {
            min: self.min.try_into()?,
            max: self.max.try_into()?,
        })
    }
}*/

/*impl<T: Into<Tp> + Copy, Tp: Copy> Into<Bounds<Tp>> for Bounds<T> {
    fn into(self) -> Bounds<Tp> {
        Bounds {
            min: self.min.into(),
            max: self.max.into(),
        }
    }
}*/
