use crate::Vector2;
use scoundrel_util::numeric::{HasOne, HasZero};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mat2<T: Copy> {
    pub col1: Vector2<T>,
    pub col2: Vector2<T>,
}

impl<T: Copy> Mat2<T> {
    pub fn from_cols(x: Vector2<T>, y: Vector2<T>) -> Self {
        Mat2 { col1: x, col2: y }
    }
    pub fn from_rows(x: Vector2<T>, y: Vector2<T>) -> Self {
        Mat2 { col1: x, col2: y }.transpose()
    }

    pub fn row_major(xx: T, xy: T, yx: T, yy: T) -> Self {
        Mat2 {
            col1: Vector2::new(xx, xy),
            col2: Vector2::new(yx, yy),
        }
    }

    pub fn transpose(&self) -> Self {
        Mat2 {
            col1: Vector2::new(self.col1.x, self.col2.x),
            col2: Vector2::new(self.col1.y, self.col2.y),
        }
    }
}

impl<T: Copy + Default> Default for Mat2<T> {
    fn default() -> Self {
        Mat2 {
            col1: Vector2::new(T::default(), T::default()),
            col2: Vector2::new(T::default(), T::default()),
        }
    }
}

impl<T: HasZero + Copy> Mat2<T> {
    pub fn zero() -> Self {
        Mat2 {
            col1: Vector2::zero(),
            col2: Vector2::zero(),
        }
    }
}

impl<T: HasZero + HasOne + Copy> Mat2<T> {
    pub fn ident() -> Self {
        Mat2 {
            col1: Vector2::new(T::one(), T::zero()),
            col2: Vector2::new(T::zero(), T::one()),
        }
    }
}

impl<T: Copy + Mul<Output = T>> Mul<T> for Mat2<T> {
    type Output = Mat2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Mat2::from_cols(self.col1 * rhs, self.col2 * rhs)
    }
}

impl<T: Copy + Div<Output = T>> Div<T> for Mat2<T> {
    type Output = Mat2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Mat2::from_cols(self.col1 / rhs, self.col2 / rhs)
    }
}

impl<T: Copy + Mul<Output = T> + Add<Output = T>> Mul<Vector2<T>> for Mat2<T> {
    type Output = Vector2<T>;

    fn mul(self, rhs: Vector2<T>) -> Self::Output {
        Vector2 {
            x: self.col1.x * rhs.x + self.col2.x * rhs.y,
            y: self.col1.y * rhs.x + self.col2.y * rhs.y,
        }
    }
}

impl<T: Copy + Mul<Output = Tp>, Tp: Sub> Mat2<T> {
    pub fn det(&self) -> <Tp as Sub>::Output {
        self.col1.x * self.col2.y - self.col2.x * self.col1.y
    }
}

impl<
        T: Copy
            + Mul<Output = T>
            + Sub<Output = T>
            + Add<Output = T>
            + Div<Output = T>
            + HasZero
            + PartialEq<T>,
    > Mat2<T>
{
    pub fn inverse(&self) -> Option<Mat2<T>> {
        let determinant = self.det();
        let zero = <T as HasZero>::zero();
        if determinant == zero {
            None
        } else {
            Some(
                Mat2::from_rows(
                    Vector2::new(self.col2.y, zero - self.col2.x),
                    Vector2::new(zero - self.col1.y, self.col1.x),
                ) / determinant,
            )
        }
    }
}
