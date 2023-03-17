use crate::Point;
use std::collections::HashMap;
use std::hash::Hash;

/// A data structure that indexes values based on their position in a 2D grid.
pub struct TileBin<T: Hash> {
    bins: HashMap<Point, Vec<T>>,
    positions: HashMap<T, Point>,
}

impl<T: Hash> Default for TileBin<T> {
    /// Creates a new TileBin with empty bins and positions maps.
    fn default() -> Self {
        Self {
            bins: HashMap::new(),
            positions: HashMap::new(),
        }
    }
}

impl<T: Hash + Eq + Clone> TileBin<T> {
    /// Inserts a value into the index at a given position.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to insert into the index.
    /// * `position` - The position at which to insert the value.
    ///
    /// # Returns
    ///
    /// `true` if the value was inserted successfully, `false` if the value was already present at the given location.
    pub fn insert(&mut self, value: T, position: Point) -> bool {
        let bin = self.bins.entry(position).or_insert_with(Vec::new);
        if bin.contains(&value) {
            return false;
        }
        self.positions.insert(value.clone(), position);
        bin.push(value);
        true
    }

    /// Removes a given value from the index.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to remove from the index.
    ///
    /// # Returns
    ///
    /// `true` if the value was found and removed, `false` otherwise.
    pub fn remove(&mut self, value: &T) -> bool {
        if let Some(position) = self.positions.get(value) {
            if let Some(bin) = self.bins.get_mut(position) {
                bin.retain(|v| v != value);
            }
        }
        self.positions.remove(value).is_some()
    }

    /// Changes the position of a given value in the index.
    pub fn relocate(&mut self, value: T, new_position: Point) {
        self.remove(&value);
        self.insert(value, new_position);
    }

    /// Remove all values from the index.
    pub fn clear(&mut self) {
        self.bins.clear();
        self.positions.clear();
    }

    /// Returns an iterator over all values associated with a given position.
    ///
    /// # Arguments
    ///
    /// * `position` - The position for which to retrieve values.
    pub fn values_at(&self, position: Point) -> impl Iterator<Item = &T> {
        self.bins
            .get(&position)
            .map(|bin| &bin[..])
            .unwrap_or(&[])
            .iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut bin = TileBin::<i32>::default();
        assert!(bin.insert(1, Point::new(0, 0)));
        assert!(bin.insert(2, Point::new(0, 0)));
        assert!(!bin.insert(1, Point::new(0, 0)));
        assert!(bin.insert(3, Point::new(1, 0)));
        assert!(bin.insert(4, Point::new(1, 1)));
        assert!(bin.insert(5, Point::new(2, 2)));
    }

    #[test]
    fn test_remove() {
        let mut bin = TileBin::<i32>::default();
        assert!(!bin.remove(&1));
        bin.insert(1, Point::new(0, 0));
        assert!(bin.remove(&1));
        assert!(!bin.remove(&1));
        bin.insert(1, Point::new(0, 0));
        bin.insert(2, Point::new(0, 0));
        assert!(bin.remove(&1));
        assert!(bin.remove(&2));
    }

    #[test]
    fn test_relocate() {
        let mut bin = TileBin::<i32>::default();
        bin.insert(1, Point::new(0, 0));
        bin.insert(2, Point::new(0, 1));
        bin.relocate(1, Point::new(1, 1));
        assert!(bin.values_at(Point::new(0, 0)).next().is_none());
        assert_eq!(bin.values_at(Point::new(1, 1)).next(), Some(&1));
        assert_eq!(bin.values_at(Point::new(0, 1)).next(), Some(&2));
    }

    #[test]
    fn test_clear() {
        let mut bin = TileBin::<i32>::default();
        bin.insert(1, Point::new(0, 0));
        bin.insert(2, Point::new(1, 1));
        bin.clear();
        assert!(bin.values_at(Point::new(0, 0)).next().is_none());
        assert!(bin.values_at(Point::new(1, 1)).next().is_none());
    }
}
