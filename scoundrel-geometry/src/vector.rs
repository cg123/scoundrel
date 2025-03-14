use std::hash::Hash;
use std::ops::{Index, IndexMut};

use paste::paste;
use scoundrel_util::ignore_ident;
use scoundrel_util::numeric::{HasSqrt, HasZero, Ring};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[cfg(feature = "tui")]
use tui::layout::Position;

macro_rules! binop_rhs {
    ($component:ident, $rhs:ident, vec) => {
        $rhs.$component
    };
    ($component:ident, $rhs:ident, scalar) => {
        $rhs
    };
}
macro_rules! binop_rhs_type {
    ($vector:ident, $T:ident, vec) => {$vector<$T>};
    ($vector:ident, $T:ident, scalar) => {$T};
}
macro_rules! impl_binop_trait {
    ($trait:ident for $struct:ident, vec {
        $($stuff:tt)*
    }) => {
        impl<T: std::ops::$trait<Output=Tp>, Tp> std::ops::$trait for $struct<T> {
            $($stuff)*
        }
    };
    ($trait:ident for $struct:ident, scalar {
        $($stuff:tt)*
    }) => {
        impl<T: std::ops::$trait<Output=Tp> + Copy, Tp> std::ops::$trait<T> for $struct<T> {
            $($stuff)*
        }
    };
}

macro_rules! vector_binary_op {
    ($struct:ident{$($component:ident),+}, $trait:ident, $op:tt, $mode:ident) => {
        impl_binop_trait!($trait for $struct, $mode {
            type Output = $struct<Tp>;
            paste! {
                fn [<$trait:snake>] (self, rhs: binop_rhs_type!($struct, T, $mode)) -> Self::Output {
                    $struct {
                        $(
                            $component: self.$component $op binop_rhs!($component, rhs, $mode),
                        )+
                    }
                }
            }
        });
    };
}

macro_rules! vector_inplace_op {
    ($struct:ident{$($component:ident),+}, $trait:ident, $op:tt, $mode:ident) => {
        impl<T: std::ops::$trait + Copy> std::ops::$trait<binop_rhs_type!($struct, T, $mode)> for $struct<T> {
            paste! {
                fn [<$trait:snake>] (&mut self, rhs: binop_rhs_type!($struct, T, $mode)) {
                    $(
                        self.$component $op binop_rhs!($component, rhs, $mode);
                    )+
                }
            }
        }
    };
}

/// A trait representing an N-dimensional vector with components of type `T`.
///
/// This trait is implemented by all vector types in this module and provides
/// common properties and conversions between vectors and tuples.
pub trait VectorN<T>:
    From<Self::Tuple> + Into<Self::Tuple> + IntoIterator<Item = T>
{
    /// The tuple type corresponding to this vector type.
    type Tuple;

    /// The dimensionality of the vector.
    const LENGTH: usize;
}

macro_rules! count_components {
    ($($component:ident),+) => {(0usize $(+ ignore_ident!($component, 1))+)};
}

macro_rules! define_vector {
    (
        $(#[$outer:meta])*
        $name:ident{$($component:ident),+}
    ) => {
        $(#[$outer])*
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        pub struct $name<T> {
            $(
                pub $component: T,
            )+
        }

        impl<T> VectorN<T> for $name<T> {
            type Tuple = ( $(ignore_ident!($component, T)),+ );
            const LENGTH: usize = count_components!( $($component),+ );
        }

        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = <Vec<T> as IntoIterator>::IntoIter;
            fn into_iter(self) -> Self::IntoIter {
                vec![$(self.$component),+].into_iter()
            }
        }

        impl<T> From< ( $(ignore_ident!($component, T)),+ ) > for $name<T> {
            fn from(tup: <$name<T> as VectorN<T>>::Tuple) -> Self {
                let ($($component),+) = tup;
                Self { $($component),+ }
            }
        }
        impl<T> From<$name<T>> for ( $(ignore_ident!($component, T)),+ ) {
            fn from(vec: $name<T>) -> Self {
                ( $(vec.$component),+ )
            }
        }

        impl<T> $name<T> {
            /// Creates a new vector with the given components.
            pub fn new($($component: T),+) -> Self {
                Self {
                    $(
                    $component,
                    )+
                }
            }

            /// Creates a new vector by applying a functor `f` to each element.
            pub fn map<F: FnMut(T) -> Tp, Tp>(self, mut f: F) -> $name<Tp> {
                $name {
                    $(
                    $component: f(self.$component),
                    )+
                }
            }
        }

        impl<T: HasZero> HasZero for $name<T> {
            fn zero() -> Self {
                Self {
                    $(
                    $component: <T as HasZero>::zero(),
                    )+
                }
            }
        }

        impl<T: HasZero> $name<T> {
            pub fn zero() -> Self {
                <Self as HasZero>::zero()
            }
        }

        impl<T: Ring + HasZero + Copy> $name<T> {
            /// Returns the dot product of this vector with another.
            pub fn dot(&self, rhs: &Self) -> T {
                <T as HasZero>::zero() $( + self.$component * rhs.$component)+
            }

            /// Returns the squared magnitude of this vector.
            pub fn sqr_magnitude(&self) -> T {
                self.dot(self)
            }
        }

        impl<T: Ring + HasZero + Copy + HasSqrt> $name<T> {
            /// Returns the magnitude of this vector.
            pub fn magnitude(&self) -> T {
                self.sqr_magnitude()._sqrt()
            }
        }

        impl<T: Ring + HasZero + Copy + HasSqrt + std::ops::Div<T, Output=Tp>, Tp> $name<T> {
            /// Returns a unit vector aligned with this one.
            pub fn normalized(&self) -> $name<Tp> {
                *self / self.magnitude()
            }
        }

        vector_binary_op!($name {$($component),+}, Add, +, vec);
        vector_binary_op!($name {$($component),+}, Sub, -, vec);
        vector_binary_op!($name {$($component),+}, Mul, *, scalar);
        vector_binary_op!($name {$($component),+}, Div, /, scalar);
        vector_inplace_op!($name {$($component),+}, AddAssign, +=, vec);
        vector_inplace_op!($name {$($component),+}, SubAssign, -=, vec);
        vector_inplace_op!($name {$($component),+}, MulAssign, *=, scalar);
        vector_inplace_op!($name {$($component),+}, DivAssign, /=, scalar);
    };
}

define_vector!(
    /// A 2D vector with x and y components.
    ///
    /// Vector2 provides basic operations for 2D vector mathematics including addition,
    /// subtraction, scalar multiplication, dot products, and normalization.
    Vector2 { x, y }
);

define_vector!(
    /// A 3D vector with x, y, and z components.
    ///
    /// Vector3 provides basic operations for 3D vector mathematics including addition,
    /// subtraction, scalar multiplication, dot products, cross products, and normalization.
    Vector3 { x, y, z }
);

define_vector!(
    /// A 4D vector with x, y, z, and w components.
    ///
    /// Vector4 provides basic operations for 4D vector mathematics including addition,
    /// subtraction, scalar multiplication, dot products, and normalization.
    Vector4 { x, y, z, w }
);

impl<T> Vector3<T> {
    pub fn from_vector2(vec: Vector2<T>, z: T) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z,
        }
    }
}

impl<T> Vector4<T> {
    pub fn from_vector3(vec: Vector3<T>, w: T) -> Self {
        Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
            w,
        }
    }
}

impl<T: Ring + std::ops::Sub<Output = T> + Copy> Vector3<T> {
    /// Returns the three-dimensional cross product of this vector with another.
    pub fn cross(&self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

macro_rules! define_axes {
    ($(#[$outer:meta])* $name:ident {$($case:ident),+}, $vector:ident) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq)]
        pub enum $name {
            $(
                $case,
            )+
        }

        impl<T> Index<$name> for $vector<T> {
            type Output = T;

            fn index(&self, index: $name) -> &Self::Output {
                paste! {
                    match index {
                        $(
                            $name::$case => &self.[<$case:lower>],
                        )+
                    }
                }
            }
        }

        impl<T> IndexMut<$name> for $vector<T> {
            fn index_mut(&mut self, index: $name) -> &mut Self::Output {
                paste! {
                    match index {
                        $(
                            $name::$case => &mut self.[<$case:lower>],
                        )+
                    }
                }
            }
        }

        impl<T: HasZero> $vector<T> {
            pub fn along_axis(axis: $name, length: T) -> Self {
                let mut res = $vector::zero();
                res[axis] = length;
                res
            }
        }
    };
}

define_axes!(
    /// Represents the axes of a 2D coordinate system (X and Y).
    ///
    /// Used for axis-specific operations and indexing on 2D vectors.
    Axis2D { X, Y }, Vector2
);

define_axes!(
    /// Represents the axes of a 3D coordinate system (X, Y, and Z).
    ///
    /// Used for axis-specific operations and indexing on 3D vectors.
    Axis3D { X, Y, Z }, Vector3
);

define_axes!(
    /// Represents the axes of a 4D coordinate system (X, Y, Z, and W).
    ///
    /// Used for axis-specific operations and indexing on 4D vectors.
    Axis4D { X, Y, Z, W }, Vector4
);

/// Errors that can occur during axis conversion operations.
///
/// This enum represents the possible errors when attempting to convert between
/// axis types of different dimensions (for example, trying to convert a Z axis
/// to a 2D axis system, which doesn't have a Z component).
#[derive(Error, Debug)]
pub enum AxisError {
    /// Error when an Axis2D value cannot be used in the current context.
    #[error("The specified Axis2D value is not valid for this operation.")]
    InvalidAxis2D(Axis2D),

    /// Error when an Axis3D value cannot be used in the current context.
    #[error("The specified Axis3D value is not valid for this operation.")]
    InvalidAxis3D(Axis3D),

    /// Error when an Axis4D value cannot be used in the current context.
    #[error("The specified Axis4D value is not valid for this operation.")]
    InvalidAxis4D(Axis4D),
}

impl TryFrom<Axis3D> for Axis2D {
    type Error = AxisError;

    fn try_from(value: Axis3D) -> Result<Self, Self::Error> {
        match value {
            Axis3D::X => Ok(Axis2D::X),
            Axis3D::Y => Ok(Axis2D::Y),
            _ => Err(AxisError::InvalidAxis3D(value)),
        }
    }
}

impl TryFrom<Axis4D> for Axis2D {
    type Error = AxisError;

    fn try_from(value: Axis4D) -> Result<Self, Self::Error> {
        match value {
            Axis4D::X => Ok(Axis2D::X),
            Axis4D::Y => Ok(Axis2D::Y),
            _ => Err(AxisError::InvalidAxis4D(value)),
        }
    }
}

impl TryFrom<Axis4D> for Axis3D {
    type Error = AxisError;

    fn try_from(value: Axis4D) -> Result<Self, Self::Error> {
        match value {
            Axis4D::X => Ok(Axis3D::X),
            Axis4D::Y => Ok(Axis3D::Y),
            Axis4D::Z => Ok(Axis3D::Z),
            _ => Err(AxisError::InvalidAxis4D(value)),
        }
    }
}

impl From<Axis2D> for Axis3D {
    fn from(axis: Axis2D) -> Axis3D {
        match axis {
            Axis2D::X => Axis3D::X,
            Axis2D::Y => Axis3D::Y,
        }
    }
}
impl From<Axis2D> for Axis4D {
    fn from(axis: Axis2D) -> Axis4D {
        match axis {
            Axis2D::X => Axis4D::X,
            Axis2D::Y => Axis4D::Y,
        }
    }
}

impl From<Axis3D> for Axis4D {
    fn from(axis: Axis3D) -> Axis4D {
        match axis {
            Axis3D::X => Axis4D::X,
            Axis3D::Y => Axis4D::Y,
            Axis3D::Z => Axis4D::Z,
        }
    }
}

impl Axis2D {
    pub fn opposite(&self) -> Self {
        match self {
            Axis2D::X => Axis2D::Y,
            Axis2D::Y => Axis2D::X,
        }
    }
}

#[cfg(feature = "tui")]
impl<T: Copy + From<u16>> From<Position> for Vector2<T> {
    fn from(pos: Position) -> Self {
        Vector2::new(pos.x.into(), pos.y.into())
    }
}

#[cfg(feature = "tui")]
impl<T: Copy + TryInto<u16>> TryFrom<Vector2<T>> for Position {
    type Error = <T as TryInto<u16>>::Error;

    fn try_from(vec: Vector2<T>) -> Result<Self, Self::Error> {
        Ok(Position {
            x: vec.x.try_into()?,
            y: vec.y.try_into()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_creation() {
        let v = Vector2::new(1, 2);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);

        let v = Vector3::new(1, 2, 3);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
        assert_eq!(v.z, 3);

        let v = Vector4::new(1, 2, 3, 4);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
        assert_eq!(v.z, 3);
        assert_eq!(v.w, 4);
    }

    #[test]
    fn test_vector_addition() {
        let v1 = Vector2::new(1, 2);
        let v2 = Vector2::new(2, 3);
        let v3 = v1 + v2;
        assert_eq!(v3.x, 3);
        assert_eq!(v3.y, 5);

        let v1 = Vector3::new(1, 2, 3);
        let v2 = Vector3::new(2, 3, 4);
        let v3 = v1 + v2;
        assert_eq!(v3.x, 3);
        assert_eq!(v3.y, 5);
        assert_eq!(v3.z, 7);
    }

    #[test]
    fn test_dot() {
        let vec1 = Vector3::new(1, 2, 3);
        let vec2 = Vector3::new(2, 3, 4);
        let res = vec1.dot(&vec2);
        assert_eq!(res, 20);
    }

    #[test]
    fn test_cross() {
        let vec1 = Vector3::new(1, 0, 0);
        let vec2 = Vector3::new(0, 1, 0);
        let res = vec1.cross(vec2);
        assert_eq!(res.x, 0);
        assert_eq!(res.y, 0);
        assert_eq!(res.z, 1);
    }

    #[test]
    fn test_map() {
        let vec = Vector4::new(1, 2, 3, 4);
        let res = vec.map(|x| x + 1);
        assert_eq!(res.x, 2);
        assert_eq!(res.y, 3);
        assert_eq!(res.z, 4);
        assert_eq!(res.w, 5);
    }

    #[test]
    fn test_tuple_roundtrip() {
        let vec = Vector4::new(1, 2, 3, 4);
        let tup: (i32, i32, i32, i32) = vec.into();
        assert_eq!(vec, tup.into());
    }

    #[test]
    fn test_axis_indexing() {
        let vec = Vector4::new(1, 2, 3, 4);
        assert_eq!(vec.x, vec[Axis4D::X]);
        assert_eq!(vec.y, vec[Axis4D::Y]);
        assert_eq!(vec.z, vec[Axis4D::Z]);
        assert_eq!(vec.w, vec[Axis4D::W]);
    }

    #[test]
    fn test_axis_mut_indexing() {
        let mut vec = Vector4::new(1, 2, 3, 4);
        vec[Axis4D::X] = 5;
        vec[Axis4D::Y] = 6;
        vec[Axis4D::Z] = 7;
        vec[Axis4D::W] = 8;
        assert_eq!(vec.x, 5);
        assert_eq!(vec.y, 6);
        assert_eq!(vec.z, 7);
        assert_eq!(vec.w, 8);
    }

    #[test]
    fn test_axis_creation() {
        let vec = Vector4::along_axis(Axis4D::X, 7);
        assert_eq!(vec.x, 7);
        assert_eq!(vec.y, 0);
        assert_eq!(vec.z, 0);
        assert_eq!(vec.w, 0);

        let vec = Vector4::along_axis(Axis4D::Y, 7);
        assert_eq!(vec.x, 0);
        assert_eq!(vec.y, 7);
        assert_eq!(vec.z, 0);
        assert_eq!(vec.w, 0);

        let vec = Vector4::along_axis(Axis4D::Z, 7);
        assert_eq!(vec.x, 0);
        assert_eq!(vec.y, 0);
        assert_eq!(vec.z, 7);
        assert_eq!(vec.w, 0);

        let vec = Vector4::along_axis(Axis4D::W, 7);
        assert_eq!(vec.x, 0);
        assert_eq!(vec.y, 0);
        assert_eq!(vec.z, 0);
        assert_eq!(vec.w, 7);
    }

    #[test]
    fn test_axis_conversion() {
        let axis2d = Axis2D::X;
        let axis3d: Axis3D = axis2d.into();
        assert_eq!(axis3d, Axis3D::X);
        let axis4d: Axis4D = axis2d.into();
        assert_eq!(axis4d, Axis4D::X);

        let axis3d = Axis3D::Y;
        let axis2d: Result<Axis2D, _> = axis3d.try_into();
        assert_eq!(axis2d.unwrap(), Axis2D::Y);
        let axis4d: Axis4D = axis3d.into();
        assert_eq!(axis4d, Axis4D::Y);

        let axis4d = Axis4D::Z;
        let axis2d: Result<Axis2D, _> = axis4d.try_into();
        match axis2d {
            Ok(_) => panic!("Expected error"),
            Err(AxisError::InvalidAxis4D(axis)) => assert_eq!(axis, Axis4D::Z),
            Err(_) => panic!("Unexpected error - expected InvalidAxis4D(Axis4D::Z)"),
        }
        let axis3d: Result<Axis3D, _> = axis4d.try_into();
        assert_eq!(axis3d.unwrap(), Axis3D::Z);
    }
}
