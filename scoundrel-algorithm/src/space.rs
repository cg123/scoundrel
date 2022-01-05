use std::hash::Hash;
use std::ops::Add;

pub trait BaseMap {
    type Coordinate: Copy + Eq + Hash;
    type Distance: Copy + Ord + Add<Output = Self::Distance> + Default;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate>;
    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance;
}
pub trait MapOf<T: Copy>: BaseMap {
    fn get(&self, point: Self::Coordinate) -> Option<T>;
}
pub trait MapOfMut<T: Copy>: MapOf<T> {
    fn get_mut(&mut self, point: Self::Coordinate) -> Option<&mut T>;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Passability {
    Passable,
    Impassable,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Opacity {
    Opaque,
    Transparent,
}
