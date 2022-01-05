use std::cmp::Ordering;

pub struct PQEntry<T, P: Ord + Copy> {
    pub value: T,
    pub priority: P,
}

impl<T, P: Ord + Copy> Eq for PQEntry<T, P> {}

impl<T, P: Ord + Copy> PartialEq<Self> for PQEntry<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<T, P: Ord + Copy> PartialOrd<Self> for PQEntry<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<T, P: Ord + Copy> Ord for PQEntry<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}
