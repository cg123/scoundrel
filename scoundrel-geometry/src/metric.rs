use scoundrel_util::numeric::{HasAbs, HasSqrt, HasZero, Ring};
use std::ops::Sub;

/// A trait for types that can compute the distance between two vectors.
pub trait VectorMetric<T, VectorT: IntoIterator<Item = T>> {
    /// Returns the distance between two vectors.
    fn distance(&self, lhs: VectorT, rhs: VectorT) -> T;
    /// Returns a monotonically increasing function of the distance between two vectors.
    /// Preserves the ordering of distances, but actual values may not be meaningful.
    /// Useful for fast distance comparisons.
    fn distance_fast_monotonic(&self, lhs: VectorT, rhs: VectorT) -> T {
        self.distance(lhs, rhs)
    }
}

pub struct Euclidean;

impl<
        T: Copy + HasZero + Ring + Sub<Output = T> + HasSqrt,
        VectorT: IntoIterator<Item = T>,
    > VectorMetric<T, VectorT> for Euclidean
{
    fn distance(&self, lhs: VectorT, rhs: VectorT) -> T {
        self.distance_fast_monotonic(lhs, rhs)._sqrt()
    }

    fn distance_fast_monotonic(&self, lhs: VectorT, rhs: VectorT) -> T {
        lhs.into_iter()
            .zip(rhs.into_iter())
            .map(|(l, r)| (l - r) * (l - r))
            .fold(T::zero(), |acc, x| acc + x)
    }
}

pub struct Manhattan;

impl<
        T: Copy + HasZero + Ring + Sub<Output = T> + HasAbs,
        VectorT: IntoIterator<Item = T>,
    > VectorMetric<T, VectorT> for Manhattan
{
    fn distance(&self, lhs: VectorT, rhs: VectorT) -> T {
        lhs.into_iter()
            .zip(rhs.into_iter())
            .map(|(l, r)| (l - r)._abs())
            .fold(T::zero(), |acc, x| acc + x)
    }
}

pub struct Chebyshev;

impl<
        T: Copy + HasZero + Ring + Sub<Output = T> + HasAbs + Ord,
        VectorT: IntoIterator<Item = T>,
    > VectorMetric<T, VectorT> for Chebyshev
{
    fn distance(&self, lhs: VectorT, rhs: VectorT) -> T {
        lhs.into_iter()
            .zip(rhs.into_iter())
            .map(|(l, r)| (l - r)._abs())
            .fold(T::zero(), |acc, x| acc.max(x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean() {
        let metric = Euclidean;
        assert_eq!(metric.distance(vec![0.0, 0.0], vec![0.0, 0.0]), 0.0);
        assert_eq!(
            metric.distance(vec![0.0, 0.0], vec![1.0, 1.0]),
            2.0_f32.sqrt()
        );
        assert_eq!(metric.distance(vec![0.0, 0.0], vec![3.0, 4.0]), 5.0);
    }

    #[test]
    fn test_manhattan() {
        let metric = Manhattan;
        assert_eq!(metric.distance(vec![0, 0], vec![0, 0]), 0);
        assert_eq!(metric.distance(vec![0, 0], vec![1, 1]), 2);
        assert_eq!(metric.distance(vec![0, 0], vec![3, 4]), 7);
    }

    #[test]
    fn test_chebyshev() {
        let metric = Chebyshev;
        assert_eq!(metric.distance(vec![0, 0], vec![0, 0]), 0);
        assert_eq!(metric.distance(vec![0, 0], vec![1, 1]), 1);
        assert_eq!(metric.distance(vec![0, 0], vec![3, 4]), 4);
    }

    #[test]
    fn test_fast_monotonic() {
        let metric = Euclidean;
        assert_eq!(
            metric.distance_fast_monotonic(vec![0.0, 0.0], vec![0.0, 0.0]),
            0.0
        );
        assert_eq!(
            metric.distance_fast_monotonic(vec![0.0, 0.0], vec![1.0, 1.0]),
            2.0_f32
        );
        assert_eq!(
            metric.distance_fast_monotonic(vec![0.0, 0.0], vec![3.0, 4.0]),
            25.0
        );
    }
}
