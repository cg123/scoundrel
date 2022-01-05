use std::cmp::Ordering;
use thiserror::Error;

pub trait HasZero {
    fn zero() -> Self;
}
pub trait HasOne {
    fn one() -> Self;
}

macro_rules! impl_has {
    ($trait:ident, [$($type:ty),+], $method:ident, $val:literal) => {
        $(
        impl $trait for $type {
            fn $method() -> Self {
                $val
            }
        }
        )+
    };
}

impl_has!(
    HasZero,
    [u8, i8, u16, i16, u32, i32, u64, i64, usize, isize],
    zero,
    0
);
impl_has!(
    HasOne,
    [u8, i8, u16, i16, u32, i32, u64, i64, usize, isize],
    one,
    1
);
impl_has!(HasZero, [f32, f64], zero, 0.0);
impl_has!(HasOne, [f32, f64], one, 1.0);

pub trait Ring:
    std::ops::Add<Self, Output = Self> + std::ops::Mul<Self, Output = Self> + Sized
{
}
impl<T> Ring for T where T: std::ops::Add<Self, Output = Self> + std::ops::Mul<Self, Output = Self> {}

pub trait HasSqrt {
    fn _sqrt(&self) -> Self;
}
impl HasSqrt for f32 {
    fn _sqrt(&self) -> Self {
        self.sqrt()
    }
}
impl HasSqrt for f64 {
    fn _sqrt(&self) -> Self {
        self.sqrt()
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct NonNaN32 {
    value: f32,
}
impl NonNaN32 {
    pub fn new(value: f32) -> Self {
        assert!(!value.is_nan());
        Self { value }
    }
}
impl From<NonNaN32> for f32 {
    fn from(x: NonNaN32) -> Self {
        x.value
    }
}

#[derive(Error, Debug)]
pub enum NonNanError {
    #[error("you had one job")]
    IsNaN,
}
impl TryFrom<f32> for NonNaN32 {
    type Error = NonNanError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_nan() {
            Err(NonNanError::IsNaN)
        } else {
            Ok(Self { value })
        }
    }
}

impl Eq for NonNaN32 {}

impl PartialOrd<Self> for NonNaN32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for NonNaN32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.partial_cmp(&other.value).unwrap()
    }
}
