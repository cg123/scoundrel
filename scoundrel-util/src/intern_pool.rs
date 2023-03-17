use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

/// An InternID represents a unique identifier for a value in an InternPool.
#[derive(Debug)]
pub struct InternID<T>(usize, PhantomData<T>);

impl<T> Clone for InternID<T> {
    fn clone(&self) -> Self {
        InternID(self.0, PhantomData)
    }
}
impl<T> Copy for InternID<T> {}

impl<T> PartialEq for InternID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for InternID<T> {}

/// An InternPool is a data structure for interning values of a certain type.
/// Each value is assigned a unique InternID, which can be used to retrieve the
/// value later.
pub struct InternPool<T> {
    values: Vec<T>,
    ids: HashMap<T, InternID<T>>,
}

impl<T: Hash + Eq + Copy> InternPool<T> {
    /// Creates a new, empty InternPool.
    pub fn new() -> InternPool<T> {
        InternPool {
            values: vec![],
            ids: HashMap::new(),
        }
    }

    /// Adds a value to the `InternPool` and returns its `InternID`.
    /// If the value is already in the pool, its existing `InternID` is returned.
    pub fn add(&mut self, value: T) -> InternID<T> {
        if let Some(id) = self.ids.get(&value) {
            *id
        } else {
            let result = InternID(self.values.len(), PhantomData::default());
            self.values.push(value);
            self.ids.insert(value, result);
            result
        }
    }

    /// Returns a reference to the value associated with the given `InternID`.
    /// If the `InternID` is invalid or has been removed from the pool, `None` is returned.
    pub fn get(&self, id: InternID<T>) -> Option<&T> {
        self.values.get(id.0)
    }
}

impl<T: Hash + Eq + Copy> Default for InternPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_get() {
        let mut pool = InternPool::new();
        let id1 = pool.add(1);
        let id2 = pool.add(2);
        let id3 = pool.add(3);

        assert_eq!(pool.get(id1), Some(&1));
        assert_eq!(pool.get(id2), Some(&2));
        assert_eq!(pool.get(id3), Some(&3));
        assert_eq!(pool.get(InternID(3, PhantomData)), None);
    }

    #[test]
    fn test_add_duplicate() {
        let mut pool = InternPool::new();
        let id1 = pool.add(1);
        let id2 = pool.add(2);
        let id1_duplicate = pool.add(1);
        let id2_duplicate = pool.add(2);

        assert_eq!(id1, id1_duplicate);
        assert_eq!(id2, id2_duplicate);
    }
}