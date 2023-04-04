use crate::{Axis2D, Bounds, Point, Vector2};

/// An orthogonal line segment represented by an axis, a start point, and a length.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct OrthoLine {
    pub axis: Axis2D,
    pub start: Point,
    pub length: i32,
}

impl OrthoLine {
    /// Returns the end point of the line.
    pub fn end(&self) -> Point {
        let mut res = self.start;
        res[self.axis] += self.length - 1;
        res
    }

    /// Checks if the given point is contained in the line.
    pub fn contains(&self, point: Point) -> bool {
        let t = (point - self.start)[self.axis];
        t >= 0 && t < self.length
    }

    /// Applies the provided function `f` to each point in the line.
    pub fn for_each<F: FnMut(Point)>(&self, mut f: F) {
        let mut pt = self.start;
        for _ in 0..self.length {
            f(pt);
            pt[self.axis] += 1;
        }
    }
}

/// A half-space represented by an axis, an offset, and a sign.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct AxialHalfSpace<T> {
    pub axis: Axis2D,
    pub offset: T,
    pub positive: bool,
}

impl<T: Ord + Copy> AxialHalfSpace<T> {
    /// Checks if the given point is contained in the half-space.
    pub fn contains(&self, point: Vector2<T>) -> bool {
        match self.positive {
            true => point[self.axis] >= self.offset,
            false => point[self.axis] < self.offset,
        }
    }

    /// Checks if the half-space intersects with the given rectangle.
    pub fn intersects_rect(&self, rect: Bounds<T>) -> bool {
        match self.positive {
            true => rect.max[self.axis] >= self.offset,
            false => rect.min[self.axis] < self.offset,
        }
    }

    /// Clips the given rectangle to the half-space, returning the clipped rectangle.
    ///
    /// Returns `None` if there is no overlap.
    pub fn clip_rect(&self, rect: Bounds<T>) -> Option<Bounds<T>> {
        let mut new_min = rect.min;
        let mut new_max = rect.max;
        if self.positive {
            new_min[self.axis] = new_min[self.axis].max(self.offset);
        } else {
            new_max[self.axis] = new_max[self.axis].min(self.offset);
        }
        if new_max[self.axis] <= new_min[self.axis] {
            None
        } else {
            Some(Bounds::with_points(new_min, new_max))
        }
    }
}

impl<T: Clone> AxialHalfSpace<T> {
    /// Returns the opposite half-space.
    pub fn opposite(&self) -> Self {
        AxialHalfSpace {
            axis: self.axis,
            offset: self.offset.clone(),
            positive: !self.positive,
        }
    }
}

impl AxialHalfSpace<i32> {
    /// Clips the given orthogonal line to the half-space, returning the clipped line.
    ///
    /// Returns `None` if the given line does not intersect this half-space.
    pub fn clip_line(&self, line: OrthoLine) -> Option<OrthoLine> {
        if self.axis != line.axis {
            if self.contains(line.start) {
                Some(line)
            } else {
                None
            }
        } else {
            let mut new_start = line.start;
            let mut new_end = line.end();
            if self.positive && new_start[self.axis] < self.offset {
                new_start[self.axis] = self.offset
            }
            if !self.positive && new_end[self.axis] >= self.offset {
                new_end[self.axis] = self.offset - 1;
            }

            if new_end[self.axis] < new_start[self.axis] {
                None
            } else {
                Some(OrthoLine {
                    axis: line.axis,
                    start: new_start,
                    length: new_end[self.axis] - new_start[self.axis] + 1,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point, Rect};

    #[test]
    fn ortholine_end() {
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 1),
            length: 4,
        };
        assert_eq!(line.end(), Point::new(4, 1));
    }

    #[test]
    fn ortholine_contains() {
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(1, 1),
            length: 3,
        };
        assert!(line.contains(Point::new(1, 1)));
        assert!(line.contains(Point::new(1, 2)));
        assert!(line.contains(Point::new(1, 3)));
        assert!(!line.contains(Point::new(1, 4)));
    }

    #[test]
    fn halfspace_contains() {
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        assert!(!halfspace.contains(Point::new(1, 0)));
        assert!(halfspace.contains(Point::new(2, 0)));
        assert!(halfspace.contains(Point::new(3, 0)));
    }

    #[test]
    fn halfspace_opposite() {
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 1,
            positive: false,
        };
        let opposite = halfspace.opposite();
        assert_eq!(opposite.axis, Axis2D::Y);
        assert_eq!(opposite.offset, 1);
        assert_eq!(opposite.positive, true);
    }

    #[test]
    fn halfspace_intersects_rect() {
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(halfspace.intersects_rect(rect));
    }

    #[test]
    fn halfspace_clip_rect() {
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        let clipped = halfspace.clip_rect(rect).unwrap();
        assert_eq!(clipped.min, Point::new(2, 0));
        assert_eq!(clipped.max, Point::new(4, 4));
    }

    #[test]
    fn halfspace_clip_line() {
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(1, 1),
            length: 3,
        };
        let clipped = halfspace.clip_line(line);
        assert!(clipped.is_none());
    }
}
