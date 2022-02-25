use scoundrel_geometry::{Grid2D, Neighbor, Point};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Add;

pub trait BaseMap {
    type Coordinate: Copy + Eq + Hash;
    type Distance: Copy + Ord + Add<Output = Self::Distance> + Default;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate>;
    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance;

    fn coordinate_map(&self) -> MapCoords<&Self> {
        MapCoords { map: &self }
    }
}
pub trait MapOf<T: Copy>: BaseMap {
    fn get(&self, point: Self::Coordinate) -> Option<T>;
}
pub trait MapOfMut<T: Copy>: MapOf<T> {
    fn get_mut(&mut self, point: Self::Coordinate) -> Option<&mut T>;
}

pub trait MappableMap<T: Copy>: MapOf<T> + Sized {
    fn map<F: Fn(T) -> Tp, Tp: Copy>(self, f: F) -> MapOfMapped<Self, T, Tp, F> {
        MapOfMapped {
            map: self,
            f,
            marker: Default::default(),
        }
    }

    fn memoize(self) -> MemoizedMap<Self, T> {
        MemoizedMap {
            map: self,
            cache: RefCell::new(HashMap::new()),
        }
    }
}
impl<Map: MapOf<T>, T: Copy> MappableMap<T> for Map {}

impl<Map: BaseMap + ?Sized> BaseMap for &Map {
    type Coordinate = Map::Coordinate;
    type Distance = Map::Distance;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        (*self).neighbors(point)
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        (*self).distance(pt0, pt1)
    }
}

impl<Map: MapOf<T> + ?Sized, T: Copy> MapOf<T> for &Map {
    fn get(&self, point: Self::Coordinate) -> Option<T> {
        (*self).get(point).to_owned()
    }
}

pub struct MapCoords<Map: BaseMap + Sized> {
    map: Map,
}
impl<Map: BaseMap> BaseMap for MapCoords<Map> {
    type Coordinate = Map::Coordinate;
    type Distance = Map::Distance;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        self.map.neighbors(point)
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        self.map.distance(pt0, pt1)
    }
}
impl<Map: BaseMap> MapOf<Map::Coordinate> for MapCoords<Map> {
    fn get(&self, point: Self::Coordinate) -> Option<Self::Coordinate> {
        Some(point)
    }
}

pub struct MapOfMapped<Map: MapOf<T> + Sized, T: Copy, Tp: Copy, F: Fn(T) -> Tp> {
    map: Map,
    f: F,
    marker: std::marker::PhantomData<(T, Tp)>,
}
impl<Map: MapOf<T> + Sized, T: Copy, Tp: Copy, F: Fn(T) -> Tp> BaseMap
    for MapOfMapped<Map, T, Tp, F>
{
    type Coordinate = Map::Coordinate;
    type Distance = Map::Distance;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        self.map.neighbors(point)
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        self.map.distance(pt0, pt1)
    }
}
impl<Map: MapOf<T> + Sized, T: Copy, Tp: Copy, F: Fn(T) -> Tp> MapOf<Tp>
    for MapOfMapped<Map, T, Tp, F>
{
    fn get(&self, point: Self::Coordinate) -> Option<Tp> {
        self.map.get(point).map(&self.f)
    }
}

pub struct MemoizedMap<Map: MapOf<T> + Sized, T: Copy>
where
    Map::Coordinate: Hash,
{
    map: Map,
    cache: RefCell<HashMap<Map::Coordinate, Option<T>>>,
}

impl<Map: MapOf<T> + Sized, T: Copy> BaseMap for MemoizedMap<Map, T> {
    type Coordinate = Map::Coordinate;
    type Distance = Map::Distance;

    fn neighbors(&self, point: Self::Coordinate) -> Vec<Self::Coordinate> {
        self.map.neighbors(point)
    }

    fn distance(&self, pt0: Self::Coordinate, pt1: Self::Coordinate) -> Self::Distance {
        self.map.distance(pt0, pt1)
    }
}

impl<Map: MapOf<T> + Sized, T: Copy> MapOf<T> for MemoizedMap<Map, T> {
    fn get(&self, point: Self::Coordinate) -> Option<T> {
        *self
            .cache
            .borrow_mut()
            .entry(point)
            .or_insert_with(|| self.map.get(point))
    }
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
        Neighbor::all()
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
