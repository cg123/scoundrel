use scoundrel_geometry::{Grid2D, MooreNeighbor, Point};
use std::hash::Hash;
use std::marker::PhantomData;
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

impl<T: Copy> BaseMap for Grid2D<T> {
    type Coordinate = Point;
    type Distance = i32;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        MooreNeighbor::all()
            .iter()
            .map(|n| point + n.offset())
            .filter(|pt| self.index(*pt).is_some())
            .collect()
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        (pt1 - pt0).sqr_magnitude()
    }
}

impl<T: Copy> MapOf<T> for Grid2D<T> {
    fn get(&self, point: Self::Coordinate) -> Option<T> {
        self.get(point).cloned()
    }
}

pub struct MapFunctorView<'a, Map: ?Sized, T, Tp, F> {
    map: &'a Map,
    functor: F,
    marker: PhantomData<(T, Tp)>,
}

impl<'a, Map: BaseMap, T, Tp, F> BaseMap for MapFunctorView<'a, Map, T, Tp, F> {
    type Coordinate = Map::Coordinate;
    type Distance = Map::Distance;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        self.map.neighbors(point)
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        self.map.distance(pt0, pt1)
    }
}

impl<'a, Map: MapOf<T>, T: Copy, Tp: Copy, F: Fn(T) -> Tp> MapOf<Tp>
    for MapFunctorView<'a, Map, T, Tp, F>
{
    fn get(&self, point: Self::Coordinate) -> Option<Tp> {
        self.map.get(point).map(&self.functor)
    }
}

pub trait TransformableMap<T> {
    fn apply<Tp, F: Fn(T) -> Tp>(
        &self,
        functor: F,
    ) -> MapFunctorView<Self, T, Tp, F>;
}
impl<T: Copy, Map: MapOf<T>> TransformableMap<T> for Map {
    fn apply<Tp, F: Fn(T) -> Tp>(
        &self,
        functor: F,
    ) -> MapFunctorView<Self, T, Tp, F> {
        MapFunctorView {
            map: self,
            functor,
            marker: Default::default(),
        }
    }
}
