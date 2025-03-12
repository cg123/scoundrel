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
        let offset = point - self.start;
        if offset[self.axis.opposite()] != 0 {
            return false;
        }
        let t = offset[self.axis];
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

        // Test Y axis
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(5, 5),
            length: 3,
        };
        assert_eq!(line.end(), Point::new(5, 7));
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

        // Test out of bounds point on other axis
        assert!(!line.contains(Point::new(2, 2)));

        // Test zero length line
        let zero_line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(10, 10),
            length: 0,
        };
        assert!(!zero_line.contains(Point::new(10, 10)));
    }

    #[test]
    fn ortholine_for_each() {
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 5),
            length: 3,
        };

        let mut visited = Vec::new();
        line.for_each(|pt| visited.push(pt));

        assert_eq!(visited.len(), 3);
        assert_eq!(visited[0], Point::new(1, 5));
        assert_eq!(visited[1], Point::new(2, 5));
        assert_eq!(visited[2], Point::new(3, 5));

        // Test Y axis
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(5, 1),
            length: 2,
        };

        let mut visited = Vec::new();
        line.for_each(|pt| visited.push(pt));

        assert_eq!(visited.len(), 2);
        assert_eq!(visited[0], Point::new(5, 1));
        assert_eq!(visited[1], Point::new(5, 2));

        // Test zero length line (should do nothing)
        let zero_line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(0, 0),
            length: 0,
        };

        let mut visited = Vec::new();
        zero_line.for_each(|pt| visited.push(pt));

        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn halfspace_contains() {
        // Test positive X half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        assert!(!halfspace.contains(Point::new(1, 0)));
        assert!(halfspace.contains(Point::new(2, 0)));
        assert!(halfspace.contains(Point::new(3, 0)));

        // Test negative X half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: false,
        };
        assert!(halfspace.contains(Point::new(1, 0)));
        assert!(!halfspace.contains(Point::new(2, 0)));
        assert!(!halfspace.contains(Point::new(3, 0)));

        // Test Y axis
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 5,
            positive: true,
        };
        assert!(!halfspace.contains(Point::new(10, 4)));
        assert!(halfspace.contains(Point::new(10, 5)));
        assert!(halfspace.contains(Point::new(10, 6)));
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

        // Check that opposite of opposite is original
        let original = opposite.opposite();
        assert_eq!(original.axis, Axis2D::Y);
        assert_eq!(original.offset, 1);
        assert_eq!(original.positive, false);
    }

    #[test]
    fn halfspace_intersects_rect() {
        // Test positive half-space with intersection
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(halfspace.intersects_rect(rect));

        // Test positive half-space with no intersection
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(!halfspace.intersects_rect(rect));

        // Test negative half-space with intersection
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 3,
            positive: false,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(halfspace.intersects_rect(rect));

        // Test negative half-space with no intersection
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 0,
            positive: false,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(!halfspace.intersects_rect(rect));
    }

    #[test]
    fn halfspace_clip_rect() {
        // Test positive X half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        let clipped = halfspace.clip_rect(rect).unwrap();
        assert_eq!(clipped.min, Point::new(2, 0));
        assert_eq!(clipped.max, Point::new(4, 4));

        // Test negative X half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 3,
            positive: false,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        let clipped = halfspace.clip_rect(rect).unwrap();
        assert_eq!(clipped.min, Point::new(0, 0));
        assert_eq!(clipped.max, Point::new(3, 4));

        // Test Y axis
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 2,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        let clipped = halfspace.clip_rect(rect).unwrap();
        assert_eq!(clipped.min, Point::new(0, 2));
        assert_eq!(clipped.max, Point::new(4, 4));

        // Test no intersection
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 10,
            positive: true,
        };
        let rect = Rect::with_points(Point::new(0, 0), Point::new(4, 4));
        assert!(halfspace.clip_rect(rect).is_none());
    }

    #[test]
    fn halfspace_clip_line_different_axis() {
        // Test positive half-space with line on different axis (outside)
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

        // Test positive half-space with line on different axis (inside)
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 2,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(3, 1),
            length: 3,
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::Y);
        assert_eq!(clipped.start, Point::new(3, 1));
        assert_eq!(clipped.length, 3);

        // Test negative half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 5,
            positive: false,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 3),
            length: 5,
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::X);
        assert_eq!(clipped.start, Point::new(1, 3));
        assert_eq!(clipped.length, 5);
    }

    #[test]
    fn halfspace_clip_line_same_axis() {
        // Test positive half-space, line partially inside
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 3,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 5),
            length: 5, // Line goes from x=1 to x=5
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::X);
        assert_eq!(clipped.start, Point::new(3, 5)); // Start moved to offset
        assert_eq!(clipped.length, 3); // 3 points: 3, 4, 5

        // Test negative half-space, line partially inside
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 4,
            positive: false,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 5),
            length: 5, // Line goes from x=1 to x=5
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::X);
        assert_eq!(clipped.start, Point::new(1, 5)); // Start unchanged
        assert_eq!(clipped.length, 3); // 3 points: 1, 2, 3

        // Test line completely outside half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 10,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(1, 5),
            length: 5,
        };
        let clipped = halfspace.clip_line(line);
        assert!(clipped.is_none());

        // Test line completely inside half-space
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 1,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(5, 5),
            length: 3,
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::X);
        assert_eq!(clipped.start, Point::new(5, 5)); // Unchanged
        assert_eq!(clipped.length, 3); // Unchanged
    }

    #[test]
    fn halfspace_clip_line_edge_cases() {
        // Test line that ends exactly at the half-space boundary (positive)
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(5, 10),
            length: 5, // Line goes from x=5 to x=9
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.start, Point::new(5, 10));
        assert_eq!(clipped.length, 5);

        // Test line that starts exactly at the half-space boundary (negative)
        let halfspace = AxialHalfSpace {
            axis: Axis2D::X,
            offset: 5,
            positive: false,
        };
        let line = OrthoLine {
            axis: Axis2D::X,
            start: Point::new(0, 10),
            length: 5, // Line goes from x=0 to x=4
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.start, Point::new(0, 10));
        assert_eq!(clipped.length, 5);

        // Test Y axis clipping
        let halfspace = AxialHalfSpace {
            axis: Axis2D::Y,
            offset: 3,
            positive: true,
        };
        let line = OrthoLine {
            axis: Axis2D::Y,
            start: Point::new(0, 2),
            length: 4, // Line goes from y=2 to y=5
        };
        let clipped = halfspace.clip_line(line).unwrap();
        assert_eq!(clipped.axis, Axis2D::Y);
        assert_eq!(clipped.start, Point::new(0, 3)); // Start moved to offset
        assert_eq!(clipped.length, 3); // 3 points: 3, 4, 5
    }
}
