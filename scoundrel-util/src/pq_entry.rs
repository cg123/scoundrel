use std::cmp::Ordering;

/// A priority queue entry with a value and a priority.
#[derive(Debug, Clone)]
pub struct PQEntry<T, P> {
    pub value: T,
    pub priority: P,
}

impl<T, P: Ord> PartialEq<Self> for PQEntry<T, P> {
    /// Returns `true` if the priorities of `self` and `other` are equal.
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T, P: Ord> Eq for PQEntry<T, P> {}

impl<T, P: Ord> PartialOrd<Self> for PQEntry<T, P> {
    /// Compares the priorities of `self` and `other`.
    ///
    /// Returns `Some(Ordering)` if the priorities are comparable, and `None`
    /// otherwise.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<T, P: Ord> Ord for PQEntry<T, P> {
    /// Compares the priorities of `self` and `other`.
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}
