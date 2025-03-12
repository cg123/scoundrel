use std::{cmp::Ordering, ops::Add};
use thiserror::Error;

/// A trait representing a numeric type with a value equivalent to zero.
pub trait HasZero {
    fn zero() -> Self;
}

/// A trait representing a numeric type with a value equivalent to one.
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

/// A trait for types that support both addition and multiplication.
pub trait Ring:
    std::ops::Add<Self, Output = Self> + std::ops::Mul<Self, Output = Self> + Sized
{
}
impl<T> Ring for T where T: std::ops::Add<Self, Output = Self> + std::ops::Mul<Self, Output = Self> {}

/// A trait for types that have a square root function.
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

/// A `NonNaN32` is a 32 bit floating point value, guaranteed to not be NaN.
///
/// Useful for ordering.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
#[repr(transparent)]
pub struct NonNaN32 {
    value: f32,
}
impl NonNaN32 {
    /// Creates a new `NonNaN32` from a floating-point value.
    ///
    /// # Arguments
    /// * `value` - The floating-point value to wrap
    ///
    /// # Returns
    /// A new `NonNaN32` containing the given value
    ///
    /// # Panics
    /// Panics if the provided value is NaN
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

impl Add for NonNaN32 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.value + other.value)
    }
}

/// Error type for operations involving `NonNaN32` values.
///
/// This enum represents the possible errors that can occur when
/// attempting to create a `NonNaN32` from a floating-point value.
#[derive(Error, Debug)]
pub enum NonNanError {
    /// Error returned when attempting to create a `NonNaN32` from a NaN value.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Mul;

    #[test]
    fn test_has_zero_integers() {
        // Test integer types
        assert_eq!(u8::zero(), 0u8);
        assert_eq!(i8::zero(), 0i8);
        assert_eq!(u16::zero(), 0u16);
        assert_eq!(i16::zero(), 0i16);
        assert_eq!(u32::zero(), 0u32);
        assert_eq!(i32::zero(), 0i32);
        assert_eq!(u64::zero(), 0u64);
        assert_eq!(i64::zero(), 0i64);
        assert_eq!(usize::zero(), 0usize);
        assert_eq!(isize::zero(), 0isize);
    }

    #[test]
    fn test_has_zero_floats() {
        // Test floating-point types
        assert_eq!(f32::zero(), 0.0f32);
        assert_eq!(f64::zero(), 0.0f64);
    }

    #[test]
    fn test_has_one_integers() {
        // Test integer types
        assert_eq!(u8::one(), 1u8);
        assert_eq!(i8::one(), 1i8);
        assert_eq!(u16::one(), 1u16);
        assert_eq!(i16::one(), 1i16);
        assert_eq!(u32::one(), 1u32);
        assert_eq!(i32::one(), 1i32);
        assert_eq!(u64::one(), 1u64);
        assert_eq!(i64::one(), 1i64);
        assert_eq!(usize::one(), 1usize);
        assert_eq!(isize::one(), 1isize);
    }

    #[test]
    fn test_has_one_floats() {
        // Test floating-point types
        assert_eq!(f32::one(), 1.0f32);
        assert_eq!(f64::one(), 1.0f64);
    }

    // Define a simple type that implements Ring trait for testing
    #[derive(Debug, PartialEq, Eq)]
    struct TestRing(i32);

    impl Add for TestRing {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            TestRing(self.0 + other.0)
        }
    }

    impl Mul for TestRing {
        type Output = Self;

        fn mul(self, other: Self) -> Self {
            TestRing(self.0 * other.0)
        }
    }

    #[test]
    fn test_ring_trait() {
        // Test that a type implementing Add and Mul impls Ring
        let a = TestRing(5);
        let b = TestRing(10);

        // Addition test
        let sum = a + b;
        assert_eq!(sum, TestRing(15));

        // Multiplication test
        let a = TestRing(3);
        let b = TestRing(4);
        let product = a * b;
        assert_eq!(product, TestRing(12));

        // Verify TestRing implements Ring (compile-time check)
        fn takes_ring<T: Ring>(_: T) {}
        takes_ring(TestRing(0));
    }

    #[test]
    fn test_has_sqrt() {
        // Test sqrt for f32
        let x: f32 = 4.0;
        assert_eq!(x._sqrt(), 2.0);

        let x: f32 = 9.0;
        assert_eq!(x._sqrt(), 3.0);

        // Test sqrt for f64
        let x: f64 = 16.0;
        assert_eq!(x._sqrt(), 4.0);

        let x: f64 = 25.0;
        assert_eq!(x._sqrt(), 5.0);
    }

    #[test]
    fn test_nonnan32_new() {
        // Test valid creation
        let n = NonNaN32::new(42.0);
        assert_eq!(f32::from(n), 42.0);

        // Test default
        let default = NonNaN32::default();
        assert_eq!(f32::from(default), 0.0);
    }

    #[test]
    #[should_panic]
    fn test_nonnan32_new_with_nan() {
        // Should panic with NaN
        NonNaN32::new(f32::NAN);
    }

    #[test]
    fn test_nonnan32_try_from() {
        // Valid conversion
        let result = NonNaN32::try_from(123.45f32);
        assert!(result.is_ok());
        assert_eq!(f32::from(result.unwrap()), 123.45);

        // NaN should fail
        let result = NonNaN32::try_from(f32::NAN);
        assert!(result.is_err());
        match result.unwrap_err() {
            NonNanError::IsNaN => {} // Expected error variant
        }
    }

    #[test]
    fn test_nonnan32_addition() {
        let a = NonNaN32::new(5.5);
        let b = NonNaN32::new(10.1);

        let sum = a + b;
        assert_eq!(f32::from(sum), 15.6);
    }

    #[test]
    fn test_nonnan32_ordering() {
        let a = NonNaN32::new(1.0);
        let b = NonNaN32::new(2.0);
        let c = NonNaN32::new(2.0);

        // PartialOrd
        assert!(a < b);
        assert!(b > a);
        assert!(b >= c);
        assert!(c <= b);

        // Ord (total ordering)
        assert_eq!(a.cmp(&b), Ordering::Less);
        assert_eq!(b.cmp(&a), Ordering::Greater);
        assert_eq!(b.cmp(&c), Ordering::Equal);

        // Create a sorted vector
        let mut values = vec![
            NonNaN32::new(5.0),
            NonNaN32::new(2.0),
            NonNaN32::new(3.0),
            NonNaN32::new(1.0),
            NonNaN32::new(4.0),
        ];

        values.sort();

        let expected: Vec<f32> = values.into_iter().map(f32::from).collect();
        assert_eq!(expected, vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn test_nonnan32_equality() {
        let a = NonNaN32::new(42.0);
        let b = NonNaN32::new(42.0);
        let c = NonNaN32::new(100.0);

        // PartialEq
        assert_eq!(a, b);
        assert_ne!(a, c);

        // Eq (no additional tests needed, just ensuring it implements Eq)
        fn takes_eq<T: Eq>(_: T) {}
        takes_eq(a);
    }
}
