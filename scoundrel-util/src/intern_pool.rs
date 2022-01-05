use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct InternID<T>(usize, PhantomData<T>);

impl<T> Clone for InternID<T> {
    fn clone(&self) -> Self {
        InternID(self.0, PhantomData)
    }
}
impl<T> Copy for InternID<T> {}

pub struct InternPool<T> {
    values: Vec<T>,
    ids: HashMap<T, InternID<T>>,
}
impl<T: Hash + Eq + Copy> InternPool<T> {
    pub fn new() -> InternPool<T> {
        InternPool {
            values: vec![],
            ids: HashMap::new(),
        }
    }
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
    pub fn get(&self, id: InternID<T>) -> Option<&T> {
        self.values.get(id.0)
    }
}

impl<T: Hash + Eq + Copy> Default for InternPool<T> {
    fn default() -> Self {
        Self::new()
    }
}
