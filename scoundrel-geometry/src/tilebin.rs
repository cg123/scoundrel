use crate::Point;
use std::collections::HashMap;
use std::hash::Hash;

pub struct TileBin<T: Hash> {
    bins: HashMap<Point, Vec<T>>,
    positions: HashMap<T, Point>,
}

impl<T: Hash> Default for TileBin<T> {
    fn default() -> Self {
        Self {
            bins: HashMap::new(),
            positions: HashMap::new(),
        }
    }
}

impl<T: Hash + Eq> TileBin<T> {
    /// Insert a value into the index at a given position.
    /// If `false`, the given value was already present at the given location.
    pub fn insert(&mut self, value: T, position: Point) -> bool {
        let bin = self.bins.entry(position).or_insert_with(Vec::new);
        if bin.contains(&value) {
            return false;
        }
        bin.push(value);
        true
    }

    /// Remove a given value from the index.
    /// Returns `true` if the given value was found and removed.
    pub fn remove(&mut self, value: &T) -> bool {
        if let Some(position) = self.positions.get(&value) {
            if let Some(bin) = self.bins.get_mut(position) {
                bin.retain(|v| v != value);
            }
        }
        self.positions.remove(value).is_some()
    }

    pub fn relocate(&mut self, value: T, new_position: Point) {
        self.remove(&value);
        self.insert(value, new_position);
    }

    /// Remove all values from the index.
    pub fn clear(&mut self) {
        self.bins.clear();
        self.positions.clear();
    }

    /// Iterate over all values associated with a given position.
    pub fn values_at(&self, position: Point) -> impl Iterator<Item = &T> {
        self.bins
            .get(&position)
            .map(|bin| &bin[..])
            .unwrap_or(&[])
            .iter()
    }
}
