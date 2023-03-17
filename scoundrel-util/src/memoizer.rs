use std::collections::HashMap;
use std::hash::Hash;

/// A memoizer is a function that stores the results of expensive function calls and returns the
/// cached result when the same inputs occur again.
pub struct Memoizer<Argument, Out, F>
where
    Argument: Hash + Eq + Clone,
    Out: Clone,
    F: FnMut(Argument) -> Out,
{
    f: F,
    cache: HashMap<Argument, Out>,
}

impl<Argument: Hash + Eq + Clone, Out: Clone, F: FnMut(Argument) -> Out>
    Memoizer<Argument, Out, F>
{
    /// Create a new memoizer that stores the results of the given function.
    pub fn new(f: F) -> Self {
        Self {
            f,
            cache: HashMap::new(),
        }
    }

    /// Get the memoized result of the function with the given argument. If the argument has been
    /// seen before, the cached result is returned; otherwise, the function is called and the result
    /// is stored in the cache before being returned.
    pub fn value(&mut self, argument: Argument) -> Out {
        self.cache
            .entry(argument.clone())
            .or_insert_with(|| (self.f)(argument))
            .clone()
    }
}
