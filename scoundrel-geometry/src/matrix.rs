use std::ops::{Add, Div, Mul, Sub};

use scoundrel_util::numeric::{HasOne, HasZero};

use crate::Vector2;

/// A `Mat2` is a 2x2 matrix with elements of type `T`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Mat2<T: Copy> {
    pub col1: Vector2<T>,
    pub col2: Vector2<T>,
}

impl<T: Copy> Mat2<T> {
    /// Creates a `Mat2` from a pair of columns.
    pub fn from_cols(col1: Vector2<T>, col2: Vector2<T>) -> Self {
        Mat2 { col1, col2 }
    }
    /// Creates a `Mat2` from a pair of rows.
    pub fn from_rows(row1: Vector2<T>, row2: Vector2<T>) -> Self {
        Mat2 {
            col1: row1,
            col2: row2,
        }
        .transpose()
    }

    /// Creates a `Mat2` from a row-major sequence of 4 elements.
    pub fn row_major(xx: T, xy: T, yx: T, yy: T) -> Self {
        Mat2 {
            col1: Vector2::new(xx, yx),
            col2: Vector2::new(xy, yy),
        }
    }

    /// Creates a `Mat2` from a row-major sequence of 4 elements.
    pub fn col_major(xx: T, yx: T, xy: T, yy: T) -> Self {
        Mat2 {
            col1: Vector2::new(xx, yx),
            col2: Vector2::new(xy, yy),
        }
    }

    /// Returns the transpose of this matrix.
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
    /// Returns a `Mat2` with all zero elements.
    pub fn zero() -> Self {
        Mat2 {
            col1: Vector2::zero(),
            col2: Vector2::zero(),
        }
    }
}

impl<T: HasZero + HasOne + Copy> Mat2<T> {
    /// Returns an identity matrix.
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
    /// Returns the determinant of this matrix.
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
    /// Returns the inverse of this matrix, if one exists.
    ///
    /// Note that this may behave poorly with near-singular floating point matrices.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mat2_creation() {
        let mat = Mat2::row_major(1.0, 2.0, 3.0, 4.0);
        assert_eq!(mat.col1.x, 1.0);
        assert_eq!(mat.col1.y, 3.0);
        assert_eq!(mat.col2.x, 2.0);
        assert_eq!(mat.col2.y, 4.0);

        let mat = Mat2::from_cols(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0));
        assert_eq!(mat.col1.x, 1.0);
        assert_eq!(mat.col1.y, 2.0);
        assert_eq!(mat.col2.x, 3.0);
        assert_eq!(mat.col2.y, 4.0);

        let mat = Mat2::from_rows(Vector2::new(1.0, 2.0), Vector2::new(3.0, 4.0));
        assert_eq!(mat.col1.x, 1.0);
        assert_eq!(mat.col1.y, 3.0);
        assert_eq!(mat.col2.x, 2.0);
        assert_eq!(mat.col2.y, 4.0);
    }

    #[test]
    fn test_mat2_operations() {
        let mat1 = Mat2::row_major(1.0, 2.0, 3.0, 4.0);
        let mat2 = mat1 * 2.0;
        assert_eq!(mat2.col1.x, 2.0);
        assert_eq!(mat2.col1.y, 6.0);
        assert_eq!(mat2.col2.x, 4.0);
        assert_eq!(mat2.col2.y, 8.0);

        let vec = Vector2::new(1.0, 2.0);
        let vec2 = mat1 * vec;
        assert_eq!(vec2.x, 5.0);
        assert_eq!(vec2.y, 11.0);

        let det = mat1.det();
        assert_eq!(det, -2.0);

        let inv = mat1.inverse().unwrap();
        assert_eq!(inv.col1.x, -2.0);
        assert_eq!(inv.col1.y, 1.5);
        assert_eq!(inv.col2.x, 1.0);
        assert_eq!(inv.col2.y, -0.5);
    }

    #[test]
    fn test_transpose() {
        let mat = Mat2::row_major(1, 2, 3, 4);
        let transposed = mat.transpose();
        assert_eq!(transposed.col1.x, mat.col1.x);
        assert_eq!(transposed.col1.y, mat.col2.x);
        assert_eq!(transposed.col2.x, mat.col1.y);
        assert_eq!(transposed.col2.y, mat.col2.y);
    }

    #[test]
    fn test_zero() {
        let mat = Mat2::<i32>::zero();
        assert_eq!(mat.col1.x, 0);
        assert_eq!(mat.col1.y, 0);
        assert_eq!(mat.col2.x, 0);
        assert_eq!(mat.col2.y, 0);
    }

    #[test]
    fn test_ident() {
        let mat = Mat2::<i32>::ident();
        assert_eq!(mat.col1.x, 1);
        assert_eq!(mat.col1.y, 0);
        assert_eq!(mat.col2.x, 0);
        assert_eq!(mat.col2.y, 1);
    }
}
