use std::collections::HashMap;
use std::hash::Hash;

pub struct Memoizer<Argument: Hash + Eq + Clone, Out: Clone, F: FnMut(Argument) -> Out> {
    f: F,
    cache: HashMap<Argument, Out>,
}

impl<Argument: Hash + Eq + Clone, Out: Clone, F: FnMut(Argument) -> Out>
    Memoizer<Argument, Out, F>
{
    pub fn new(f: F) -> Self {
        Self {
            f,
            cache: HashMap::new(),
        }
    }

    pub fn value(&mut self, argument: Argument) -> Out {
        self.cache
            .entry(argument.clone())
            .or_insert_with(|| (self.f)(argument))
            .clone()
    }
}
