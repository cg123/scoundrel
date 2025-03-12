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
    use std::collections::HashSet;

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

    #[test]
    fn test_values_at_multiple_items() {
        let mut bin = TileBin::<i32>::default();

        // Insert multiple values at the same position
        bin.insert(1, Point::new(5, 5));
        bin.insert(2, Point::new(5, 5));
        bin.insert(3, Point::new(5, 5));

        // Collect values into a set to compare regardless of order
        let values: HashSet<i32> = bin.values_at(Point::new(5, 5)).cloned().collect();

        let expected: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
        assert_eq!(values, expected);

        // Check count
        assert_eq!(bin.values_at(Point::new(5, 5)).count(), 3);
    }

    #[test]
    fn test_values_at_empty_location() {
        let bin = TileBin::<i32>::default();

        // Query an empty location
        let values: Vec<&i32> = bin.values_at(Point::new(10, 10)).collect();
        assert!(values.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_bin() {
        let mut bin = TileBin::<i32>::default();
        bin.insert(1, Point::new(0, 0));

        // Remove item, then try to access its original position
        bin.remove(&1);
        let values: Vec<&i32> = bin.values_at(Point::new(0, 0)).collect();
        assert!(values.is_empty());

        // Verify internal hashmap cleanup
        assert!(!bin.positions.contains_key(&1));
        assert!(bin
            .bins
            .get(&Point::new(0, 0))
            .map_or(true, |v| v.is_empty()));
    }

    #[test]
    fn test_relocate_to_same_position() {
        let mut bin = TileBin::<i32>::default();
        bin.insert(1, Point::new(3, 3));

        // Relocate to the same position
        bin.relocate(1, Point::new(3, 3));

        // Should still be there
        assert_eq!(bin.values_at(Point::new(3, 3)).next(), Some(&1));
    }

    #[test]
    fn test_relocate_with_existing_items() {
        let mut bin = TileBin::<i32>::default();

        // Setup initial state
        bin.insert(1, Point::new(0, 0));
        bin.insert(2, Point::new(1, 1));
        bin.insert(3, Point::new(1, 1));

        // Relocate to a position that already has items
        bin.relocate(1, Point::new(1, 1));

        // Check original position is empty
        assert!(bin.values_at(Point::new(0, 0)).next().is_none());

        // Check all items at new position
        let values: HashSet<i32> = bin.values_at(Point::new(1, 1)).cloned().collect();

        let expected: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
        assert_eq!(values, expected);
    }

    #[test]
    fn test_custom_type() {
        #[derive(Hash, Eq, PartialEq, Clone, Debug)]
        struct Entity {
            id: u32,
            name: String,
        }

        let mut bin = TileBin::<Entity>::default();

        let e1 = Entity {
            id: 1,
            name: "Player".to_string(),
        };
        let e2 = Entity {
            id: 2,
            name: "Enemy".to_string(),
        };

        bin.insert(e1.clone(), Point::new(10, 10));
        bin.insert(e2.clone(), Point::new(20, 20));

        // Check entities are at the correct positions
        assert_eq!(bin.values_at(Point::new(10, 10)).next().unwrap().id, 1);
        assert_eq!(bin.values_at(Point::new(20, 20)).next().unwrap().id, 2);

        // Relocate one entity
        bin.relocate(e1.clone(), Point::new(15, 15));

        // Verify relocation
        assert!(bin.values_at(Point::new(10, 10)).next().is_none());
        assert_eq!(bin.values_at(Point::new(15, 15)).next().unwrap().id, 1);
    }

    #[test]
    fn test_remove_last_item_in_bin() {
        let mut bin = TileBin::<i32>::default();

        // Insert multiple items at different positions
        bin.insert(1, Point::new(0, 0));
        bin.insert(2, Point::new(1, 1));
        bin.insert(3, Point::new(2, 2));

        // Remove the only item at a position
        bin.remove(&2);

        // Check it's gone
        assert!(bin.values_at(Point::new(1, 1)).next().is_none());

        // Check other positions still have their items
        assert_eq!(bin.values_at(Point::new(0, 0)).next(), Some(&1));
        assert_eq!(bin.values_at(Point::new(2, 2)).next(), Some(&3));
    }
}
