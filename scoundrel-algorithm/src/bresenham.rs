use scoundrel_geometry::Point;

/// A Bresenham line iterator for iterating over the points on a line between two `Point`s.
pub struct Bresenham {
    /// The absolute difference between the start and end points in both dimensions.
    delta: Point,
    /// The directions to step in (either +1 or -1 on each axis), or None if iteration
    /// has concluded
    step: Option<Point>,
    /// The current error value for the line.
    error: i32,

    /// The current point in the line iteration.
    current: Point,
    /// The final point in the line.
    end: Point,
}

impl Iterator for Bresenham {
    type Item = Point;

    /// Returns the next point on the line, or `None` if the end point has been reached.
    fn next(&mut self) -> Option<Self::Item> {
        let step = self.step?;
        let point = self.current;
        if point == self.end {
            self.step = None;
            return Some(point);
        }

        let e2 = self.error * 2;
        if e2 >= self.delta.y {
            self.error += self.delta.y;
            self.current.x += step.x;
        }
        if e2 <= self.delta.x {
            self.error += self.delta.x;
            self.current.y += step.y;
        }

        Some(point)
    }
}

impl Bresenham {
    /// Creates a new `Bresenham` line iterator that iterates over the points on a line between `pt0` and `pt1`.
    ///
    /// # Examples
    ///
    /// ```
    /// use scoundrel_algorithm::Bresenham;
    /// use scoundrel_geometry::Point;
    ///
    /// let line: Vec<Point> = Bresenham::new(Point::new(0, 0), Point::new(3, 2)).collect();
    /// assert_eq!(line, vec![Point::new(0, 0), Point::new(1, 1), Point::new(2, 1), Point::new(3, 2)]);
    /// ```
    pub fn new(pt0: Point, pt1: Point) -> Bresenham {
        let delta = Point::new((pt1.x - pt0.x).abs(), -(pt1.y - pt0.y).abs());
        let step = Point::new(
            if pt0.x < pt1.x { 1 } else { -1 },
            if pt0.y < pt1.y { 1 } else { -1 },
        );
        Bresenham {
            delta,
            step: Some(step),
            error: delta.x + delta.y,
            current: pt0,
            end: pt1,
        }
    }
}
