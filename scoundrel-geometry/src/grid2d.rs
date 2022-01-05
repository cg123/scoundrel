use crate::{Neighbor, Point, Rect, Vector2};
use scoundrel_util::numeric::HasSqrt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct Grid2D<T> {
    pub data: Vec<T>,
    _width: i32,
    _height: i32,
}

impl<T: Copy> Grid2D<T> {
    pub fn new(width: i32, height: i32, fill: T) -> Grid2D<T> {
        Grid2D {
            data: vec![fill; width as usize * height as usize],
            _width: width,
            _height: height,
        }
    }
    pub fn from_iter<I: Iterator<Item = T>>(iter: I, width: i32, height: i32) -> Grid2D<T> {
        let data = iter.collect::<Vec<_>>();
        assert_eq!(
            data.len(),
            (width as usize) * (height as usize),
            "array size for 2d grid must be product of dimensions"
        );
        Grid2D {
            data,
            _width: width,
            _height: height,
        }
    }

    pub fn like<P: Copy>(other: &Grid2D<P>, fill: T) -> Grid2D<T> {
        Grid2D::new(other._width, other._height, fill)
    }

    pub fn rect(&self) -> Rect {
        Rect::with_points(Point::zero(), self.size())
    }

    pub fn size(&self) -> Point {
        Point::new(self._width, self._height)
    }
    pub fn width(&self) -> i32 {
        self._width
    }
    pub fn height(&self) -> i32 {
        self._height
    }

    pub fn index(&self, pt: Point) -> Option<usize> {
        if pt.x >= 0 && pt.x < self._width && pt.y >= 0 && pt.y < self._height {
            Some((pt.y as usize * self._width as usize) + pt.x as usize)
        } else {
            None
        }
    }
    pub fn get(&self, pt: Point) -> Option<&T> {
        self.index(pt).map(|idx| &self.data[idx])
    }
    pub fn get_mut(&mut self, pt: Point) -> Option<&mut T> {
        if let Some(idx) = self.index(pt) {
            Some(&mut self.data[idx])
        } else {
            None
        }
    }

    pub fn set(&mut self, pt: Point, value: T) -> bool {
        match self.index(pt) {
            Some(idx) => {
                self.data[idx] = value;
                true
            }
            None => false,
        }
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32, fill: T) {
        self.data.clear();
        self._width = new_width;
        self._height = new_height;
        self.data
            .resize(new_width as usize * new_height as usize, fill);
    }

    pub fn clear(&mut self, fill: T) {
        for v in &mut self.data {
            *v = fill;
        }
    }

    pub fn copy_subregion(&self, region: Rect) -> Grid2D<T> {
        let x0 = max(0, region.min.x);
        let y0 = max(0, region.min.y);
        let x1 = min(self._width, region.max.x);
        let y1 = min(self._height, region.max.y);
        let wp = x1 - x0;
        let hp = y1 - y0;
        let mut result = Grid2D::new(wp, hp, self.data[0]);
        for y in y0..y1 {
            for x in x0..x1 {
                result.set(
                    Point::new(x - x0, y - y0),
                    *self.get(Point::new(x, y)).unwrap(),
                );
            }
        }
        result
    }

    pub fn map<P: Copy, F: FnMut(&T) -> P>(&self, func: F) -> Grid2D<P> {
        Grid2D::from_iter(self.data.iter().map(func), self._width, self._height)
    }

    pub fn map_coords<P: Copy, F: FnMut(Point) -> P>(&self, f: &mut F) -> Grid2D<P> {
        let data = (0..self.data.len()).map(|idx| {
            let x = idx % (self._width as usize);
            let y = idx / (self._width as usize);
            f(Point::new(x as i32, y as i32))
        });
        Grid2D::from_iter(data, self._width, self._height)
    }

    pub fn iter_coords(&self) -> GridCoordIterator {
        GridCoordIterator {
            current: Point::zero(),
            max: self.size(),
        }
    }
    pub fn iter(&self) -> GridIterator<T> {
        GridIterator {
            grid: self,
            ci: self.iter_coords(),
        }
    }
    pub fn iter_mut(&mut self) -> GridIteratorMut<T> {
        let ci = self.iter_coords();
        GridIteratorMut { grid: self, ci }
    }

    pub fn iter_neighborhoods(&self) -> GridNeighborhoodIterator<T> {
        GridNeighborhoodIterator {
            grid: self,
            ci: self.iter_coords(),
        }
    }
}

impl<
        T: Copy + std::ops::Mul<f32, Output = Tp>,
        Tp: std::ops::Mul<f32, Output = Tp> + std::ops::Add<Output = Tp>,
    > Grid2D<T>
{
    pub fn interpolate(&self, mut pt: Vector2<f32>) -> Tp {
        pt.x = pt.x.clamp(0.0, self._width as f32 - 1.0);
        pt.y = pt.y.clamp(0.0, self._height as f32 - 1.0);

        let pt0 = pt.map(|c| c.floor() as i32);
        let frac = pt - pt0.map(|c| c as f32);
        let x0y0 = *self.get(pt0).unwrap();
        let val_at = |pp| *self.get(pp).unwrap_or(&x0y0);
        let x1y0 = val_at(pt0 + Point::new(1, 0));
        let x1y1 = val_at(pt0 + Point::new(1, 1));
        let x0y1 = val_at(pt0 + Point::new(0, 1));

        let y0 = x0y0 * (1.0 - frac.x) + x1y0 * frac.x;
        let y1 = x0y1 * (1.0 - frac.x) + x1y1 * frac.x;
        y0 * (1.0 - frac.y) + y1 * frac.y
    }
}

impl<
        T: Copy + HasSqrt + std::ops::Sub<Output = T> + std::ops::Div<Output = Tp> + From<i8>,
        Tp: Copy + From<i8>,
    > Grid2D<T>
{
    pub fn gradient(&self, at: Point) -> Option<Vector2<Tp>> {
        let center_value = match self.get(at) {
            Some(&value) => value,
            None => return None,
        };
        let neighbor_or_center = |n: Neighbor| {
            if let Some(&value) = self.get(at + n.offset()) {
                (1, value)
            } else {
                (0, center_value)
            }
        };
        let grad = |n: Neighbor| {
            let (d_pos, e_pos) = neighbor_or_center(n);
            let (d_neg, e_neg) = neighbor_or_center(n.opposite());
            if d_pos + d_neg > 0 {
                (e_pos - e_neg) / ((d_pos + d_neg) as i8).into()
            } else {
                0.into()
            }
        };
        let dv_dx = grad(Neighbor::Right);
        let dv_dy = grad(Neighbor::Up);
        Some(Vector2::new(dv_dx, dv_dy))
    }
}

pub struct GridCoordIterator {
    current: Point,
    max: Point,
}
impl Iterator for GridCoordIterator {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.y >= self.max.y {
            None
        } else {
            let pt = self.current;
            self.current.x += 1;
            if self.current.x >= self.max.x {
                self.current.x = 0;
                self.current.y += 1;
            }
            Some(pt)
        }
    }
}

pub struct GridIterator<'a, T> {
    grid: &'a Grid2D<T>,
    ci: GridCoordIterator,
}
impl<'a, T: Copy> Iterator for GridIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.ci.next().and_then(|pt| self.grid.get(pt))
    }
}

pub struct GridIteratorMut<'a, T> {
    grid: &'a mut Grid2D<T>,
    ci: GridCoordIterator,
}
impl<'a, T: Copy> Iterator for GridIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next<'n>(&'n mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(pt) = self.ci.next() {
                let value: &'n mut T = self.grid.get_mut(pt).unwrap();
                Some(std::mem::transmute(value))
            } else {
                None
            }
        }
    }
}

pub struct GridNeighborhoodIterator<'a, T> {
    grid: &'a Grid2D<T>,
    ci: GridCoordIterator,
}
impl<'a, T: Copy> Iterator for GridNeighborhoodIterator<'a, T> {
    type Item = (T, [Option<T>; 8]);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pt) = self.ci.next() {
            let v0 = *self.grid.get(pt).unwrap();
            let mut neighbors = [None; 8];
            for n in Neighbor::all() {
                neighbors[n.to_index()] = self.grid.get(pt + n.offset()).copied();
            }
            Some((v0, neighbors))
        } else {
            None
        }
    }
}
