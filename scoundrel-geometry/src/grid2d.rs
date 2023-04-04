use crate::{MooreNeighbor, Point, Rect, Vector2};
use scoundrel_util::numeric::HasSqrt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A 2D grid data structure.
///
/// The grid is represented as a contiguous 1D vector with dimensions `width` by `height`. The
/// element at `(x, y)` can be accessed using the index `(y * width + x) as usize`.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub struct Grid2D<T> {
    pub data: Vec<T>,
    _width: i32,
    _height: i32,
}

impl<T: Copy> Grid2D<T> {
    /// Creates a new grid of the specified dimensions, filled with the given value.
    pub fn new(width: i32, height: i32, fill: T) -> Grid2D<T> {
        Grid2D {
            data: vec![fill; width as usize * height as usize],
            _width: width,
            _height: height,
        }
    }

    /// Creates a new grid with the same dimensions as another grid, filled with the given value.
    pub fn like<P: Copy>(other: &Grid2D<P>, fill: T) -> Grid2D<T> {
        Grid2D::new(other._width, other._height, fill)
    }

    /// Resizes the `Grid2D` to the given dimensions and fills with the given value.
    pub fn resize(&mut self, new_width: i32, new_height: i32, fill: T) {
        self.data.clear();
        self._width = new_width;
        self._height = new_height;
        self.data
            .resize(new_width as usize * new_height as usize, fill);
    }

    /// Resets all values in the grid to a given fill value.
    pub fn clear(&mut self, fill: T) {
        for v in &mut self.data {
            *v = fill;
        }
    }
}

impl<T> Grid2D<T> {
    /// Creates a new grid from an iterator.
    ///
    /// The iterator should yield exactly `width * height` elements. If it yields more or less, this method will panic.
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

    /// Returns the rectangular bounds of the grid.
    pub fn rect(&self) -> Rect {
        Rect::with_points(Point::zero(), self.size())
    }

    /// Returns the size of the grid.
    pub fn size(&self) -> Point {
        Point::new(self._width, self._height)
    }
    /// Returns the width of the grid.
    pub fn width(&self) -> i32 {
        self._width
    }

    /// Returns the height of the grid.
    pub fn height(&self) -> i32 {
        self._height
    }

    /// Returns the index into `data` for the element at `(x, y)`.
    ///
    /// Returns `None` if the index is out of bounds.
    pub fn index(&self, pt: Point) -> Option<usize> {
        if pt.x >= 0 && pt.x < self._width && pt.y >= 0 && pt.y < self._height {
            Some((pt.y as usize * self._width as usize) + pt.x as usize)
        } else {
            None
        }
    }

    /// Returns a reference to the element at the given point, if it is within the bounds of the `Grid2D`.
    pub fn get(&self, pt: Point) -> Option<&T> {
        self.index(pt).map(|idx| &self.data[idx])
    }

    /// Returns a mutable reference to the element at the given point, if it is within the bounds of the `Grid2D`.
    pub fn get_mut(&mut self, pt: Point) -> Option<&mut T> {
        if let Some(idx) = self.index(pt) {
            Some(&mut self.data[idx])
        } else {
            None
        }
    }

    pub fn set(&mut self, pt: Point, value: T) -> bool {
        match self.get_mut(pt) {
            Some(val) => {
                *val = value;
                true
            }
            _ => false,
        }
    }

    /// Applies a function to each element of the grid and returns a new grid with the results.
    /// The function `f` should take an element of the grid as its argument and return a new value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use scoundrel_geometry::{Grid2D, Point};
    /// let grid = Grid2D::new(3, 3, 0);
    /// let new_grid = grid.map(|x| x + 1);
    /// assert_eq!(new_grid.get(Point::new(1, 1)), Some(&1));
    /// ```
    pub fn map<F: FnMut(&T) -> P, P>(&self, func: F) -> Grid2D<P> {
        Grid2D::from_iter(self.data.iter().map(func), self._width, self._height)
    }

    /// Maps the given function over the coordinates of the grid and returns a new grid with the
    /// result of the function at each coordinate.
    pub fn map_coords<P: Copy, F: FnMut(Point) -> P>(&self, mut f: F) -> Grid2D<P> {
        let data = (0..self.data.len()).map(|idx| {
            let x = idx % (self._width as usize);
            let y = idx / (self._width as usize);
            f(Point::new(x as i32, y as i32))
        });
        Grid2D::from_iter(data, self._width, self._height)
    }

    /// Returns an iterator over the coordinates of all cells in the grid.
    ///
    /// This method creates an iterator that returns each coordinate in the grid, starting at the
    /// top-left corner and moving row by row until the bottom-right corner is reached.
    ///
    /// # Examples
    ///
    /// ```
    /// # use scoundrel_geometry::{Grid2D, Point};
    /// let grid = Grid2D::new(3, 2, 0);
    /// let mut coords = grid.iter_coords();
    ///
    /// assert_eq!(coords.next(), Some(Point::new(0, 0)));
    /// assert_eq!(coords.next(), Some(Point::new(1, 0)));
    /// assert_eq!(coords.next(), Some(Point::new(2, 0)));
    /// assert_eq!(coords.next(), Some(Point::new(0, 1)));
    /// assert_eq!(coords.next(), Some(Point::new(1, 1)));
    /// assert_eq!(coords.next(), Some(Point::new(2, 1)));
    /// assert_eq!(coords.next(), None);
    /// ```
    ///
    /// # Notes
    ///
    /// This method returns an iterator that yields coordinates in `(x, y)` order, where `x` is
    /// the column index and `y` is the row index. This order is consistent with the `(width, height)`
    /// convention used when creating the grid.
    pub fn iter_coords(&self) -> GridCoordIterator {
        GridCoordIterator {
            current: Point::zero(),
            max: self.size(),
        }
    }

    /// Returns an iterator over the values of the `Grid2D`.
    ///
    /// The iterator will visit each element in row-major order, i.e., it will visit all
    /// elements in the first row from left to right, then all elements in the second row
    /// from left to right, and so on.
    pub fn iter(&self) -> GridIterator<T> {
        GridIterator {
            grid: self,
            ci: self.iter_coords(),
        }
    }

    /// Returns an iterator that yields the moore neighborhood of each coordinate in the grid.
    ///
    /// The iterator will visit each coordinate in the grid and return a `(T, [Option<T>; 8])`
    /// representing the value at each location and at each `MooreNeighbor`. The index in
    /// the array corresponds to the `MooreNeighbor` enum value.
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
    /// Performs bilinear sampling on the 2D grid using the given floating point
    /// coordinates.
    ///
    /// Returns the interpolated value at the given coordinates using bilinear
    /// interpolation. If the given coordinates are outside the bounds of the grid,
    /// this function returns None.
    pub fn interpolate(&self, mut pt: Vector2<f32>) -> Option<Tp> {
        pt.x = pt.x.clamp(0.0, self._width as f32 - 1.0);
        pt.y = pt.y.clamp(0.0, self._height as f32 - 1.0);

        let pt0 = pt.map(|c| c.floor() as i32);
        let frac = pt - pt0.map(|c| c as f32);
        let x0y0 = *self.get(pt0)?;
        let val_at = |pp| *self.get(pp).unwrap_or(&x0y0);
        let x1y0 = val_at(pt0 + Point::new(1, 0));
        let x1y1 = val_at(pt0 + Point::new(1, 1));
        let x0y1 = val_at(pt0 + Point::new(0, 1));

        let y0 = x0y0 * (1.0 - frac.x) + x1y0 * frac.x;
        let y1 = x0y1 * (1.0 - frac.x) + x1y1 * frac.x;
        Some(y0 * (1.0 - frac.y) + y1 * frac.y)
    }
}

impl<
        T: Copy + HasSqrt + std::ops::Sub<Output = T> + std::ops::Div<Output = Tp> + From<i8>,
        Tp: Copy + From<i8>,
    > Grid2D<T>
{
    /// Computes a central difference approximation of the gradient at the given 2D
    /// position.
    ///
    /// This method returns the gradient vector at the given `position`, which
    /// represents the rate of change of the stored value with respect to the X and Y axes.
    pub fn gradient(&self, at: Point) -> Option<Vector2<Tp>> {
        let center_value = match self.get(at) {
            Some(&value) => value,
            None => return None,
        };
        let neighbor_or_center = |n: MooreNeighbor| {
            if let Some(&value) = self.get(at + n.offset()) {
                (1, value)
            } else {
                (0, center_value)
            }
        };
        let grad = |n: MooreNeighbor| {
            let (d_pos, e_pos) = neighbor_or_center(n);
            let (d_neg, e_neg) = neighbor_or_center(n.opposite());
            if d_pos + d_neg > 0 {
                (e_pos - e_neg) / ((d_pos + d_neg) as i8).into()
            } else {
                0.into()
            }
        };
        let dv_dx = grad(MooreNeighbor::Right);
        let dv_dy = grad(MooreNeighbor::Up);
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
            for n in MooreNeighbor::all() {
                neighbors[n.to_index()] = self.grid.get(pt + n.offset()).copied();
            }
            Some((v0, neighbors))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let grid = Grid2D::new(2, 3, 0);
        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.get(Point::new(0, 0)), Some(&0));
        assert_eq!(grid.get(Point::new(1, 2)), Some(&0));
        assert_eq!(grid.get(Point::new(-1, 0)), None);
        assert_eq!(grid.get(Point::new(2, 2)), None);
    }

    #[test]
    fn test_like() {
        let grid1 = Grid2D::new(3, 3, 1);
        let grid2 = Grid2D::like(&grid1, 0);
        assert_eq!(grid2.width(), 3);
        assert_eq!(grid2.height(), 3);
        assert_eq!(grid2.get(Point::new(0, 0)), Some(&0));
        assert_eq!(grid2.get(Point::new(1, 2)), Some(&0));
        assert_eq!(grid2.get(Point::new(-1, 0)), None);
        assert_eq!(grid2.get(Point::new(2, 3)), None);
    }

    #[test]
    fn test_resize() {
        let mut grid = Grid2D::new(2, 2, 0);
        grid.resize(3, 3, 1);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.get(Point::new(0, 0)), Some(&1));
        assert_eq!(grid.get(Point::new(2, 2)), Some(&1));
        assert_eq!(grid.get(Point::new(-1, 0)), None);
        assert_eq!(grid.get(Point::new(3, 3)), None);
    }

    #[test]
    fn test_clear() {
        let mut grid = Grid2D::new(2, 3, 1);
        grid.clear(7);
        assert_eq!(grid.width(), 2);
        assert_eq!(grid.height(), 3);
        assert_eq!(grid.get(Point::new(0, 0)), Some(&7));
        assert_eq!(grid.get(Point::new(1, 2)), Some(&7));
    }

    #[test]
    fn test_from_iter() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let grid = Grid2D::from_iter(data.iter().cloned(), 3, 2);
        assert_eq!(grid.width(), 3);
        assert_eq!(grid.height(), 2);
        assert_eq!(grid.get(Point::new(0, 0)), Some(&1));
        assert_eq!(grid.get(Point::new(2, 1)), Some(&6));
    }

    #[test]
    fn test_rect() {
        let grid = Grid2D::new(3, 4, 0);
        let rect = grid.rect();
        assert_eq!(rect.min, Point::zero());
        assert_eq!(rect.max, Point::new(3, 4));
    }

    #[test]
    fn test_size() {
        let grid = Grid2D::new(2, 5, 0);
        assert_eq!(grid.size(), Point::new(2, 5));
    }

    #[test]
    fn test_index() {
        let grid = Grid2D::new(3, 3, 0);
        assert_eq!(grid.index(Point::new(1, 1)), Some(4));
        assert_eq!(grid.index(Point::new(-1, 0)), None);
    }
}
