use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

pub struct PointerEqual<T>(pub T)
where
    T: ?Sized + Deref;
impl<T> PointerEqual<T>
where
    T: ?Sized + Deref,
{
    fn address(&self) -> *const T::Target {
        &*self.0
    }
}

impl<T> Hash for PointerEqual<T>
where
    T: ?Sized + Deref,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address().hash(state)
    }
}

impl<T> PartialEq for PointerEqual<T>
where
    T: ?Sized + Deref,
{
    fn eq(&self, other: &Self) -> bool {
        self.address() == other.address()
    }
}
impl<T> Eq for PointerEqual<T> where T: ?Sized + Deref {}
impl<T> PartialOrd for PointerEqual<T>
where
    T: ?Sized + Deref,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.address().partial_cmp(&other.address())
    }
}
impl<T> Ord for PointerEqual<T>
where
    T: ?Sized + Deref,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.address().cmp(&other.address())
    }
}

impl<T> Deref for PointerEqual<T>
where
    T: ?Sized + Deref,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for PointerEqual<T>
where
    T: ?Sized + DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
