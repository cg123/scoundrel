use paste::paste;
use scoundrel_util::ignore_ident;
use scoundrel_util::numeric::{HasSqrt, HasZero, Ring};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::ops::{Index, IndexMut};
use thiserror::Error;

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

pub trait VectorN<T>: From<Self::Tuple> + Into<Self::Tuple> + IntoIterator<Item = T> {
    type Tuple;
    const LENGTH: usize;
}

macro_rules! count_components {
    ($($component:ident),+) => {(0usize $(+ ignore_ident!($component, 1))+)};
}

macro_rules! define_vector {
    ($name:ident{$($component:ident),+}) => {
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
            pub fn new($($component: T),+) -> Self {
                Self {
                    $(
                    $component,
                    )+
                }
            }

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

        impl<T: Copy> $name<T> {
            pub fn to_tuple(self) -> ( $(ignore_ident!($component, T)),+ ) {
                ($(self.$component),+)
            }
        }

        impl<T: Ring + HasZero + Copy> $name<T> {
            pub fn dot(&self, rhs: &Self) -> T {
                <T as HasZero>::zero() $( + self.$component * rhs.$component)+
            }
            pub fn sqr_magnitude(&self) -> T {
                self.dot(self)
            }
        }

        impl<T: Ring + HasZero + Copy + HasSqrt> $name<T> {
            pub fn magnitude(&self) -> T {
                self.sqr_magnitude()._sqrt()
            }
        }

        impl<T: Ring + HasZero + Copy + HasSqrt + std::ops::Div<T, Output=Tp>, Tp> $name<T> {
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

define_vector!(Vector2 { x, y });
define_vector!(Vector3 { x, y, z });
define_vector!(Vector4 { x, y, z, w });

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
    pub fn cross(&self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

macro_rules! define_axis {
    ($name:ident {$($case:ident),+}, $vector:ident) => {
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
define_axis!(Axis2D { X, Y }, Vector2);
define_axis!(Axis3D { X, Y, Z }, Vector3);
define_axis!(Axis4D { X, Y, Z, W }, Vector4);

#[derive(Error, Debug)]
pub enum AxisError {
    #[error("The specified Axis2D value is not valid for this operation.")]
    InvalidAxis2D(Axis2D),
    #[error("The specified Axis3D value is not valid for this operation.")]
    InvalidAxis3D(Axis3D),
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
