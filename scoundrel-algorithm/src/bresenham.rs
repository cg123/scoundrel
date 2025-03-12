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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_basic() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(0, 0), Point::new(3, 2)).collect();
        assert_eq!(
            line,
            vec![
                Point::new(0, 0),
                Point::new(1, 1),
                Point::new(2, 1),
                Point::new(3, 2)
            ]
        );
    }

    #[test]
    fn test_line_reverse() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(3, 2), Point::new(0, 0)).collect();
        assert_eq!(
            line,
            vec![
                Point::new(3, 2),
                Point::new(2, 1),
                Point::new(1, 1),
                Point::new(0, 0)
            ]
        );
    }

    #[test]
    fn test_horizontal_line() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(0, 0), Point::new(5, 0)).collect();
        assert_eq!(
            line,
            vec![
                Point::new(0, 0),
                Point::new(1, 0),
                Point::new(2, 0),
                Point::new(3, 0),
                Point::new(4, 0),
                Point::new(5, 0)
            ]
        );
    }

    #[test]
    fn test_vertical_line() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(0, 0), Point::new(0, 5)).collect();
        assert_eq!(
            line,
            vec![
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(0, 2),
                Point::new(0, 3),
                Point::new(0, 4),
                Point::new(0, 5)
            ]
        );
    }

    #[test]
    fn test_steep_line() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(0, 0), Point::new(2, 7)).collect();
        // Verify line properties for steep slope
        assert_eq!(line[0], Point::new(0, 0));
        assert_eq!(line[line.len() - 1], Point::new(2, 7));
        assert_eq!(line.len(), 8); // Start, end, and 6 points between
    }

    #[test]
    fn test_single_point() {
        let point = Point::new(2, 3);
        let line: Vec<Point> = Bresenham::new(point, point).collect();
        assert_eq!(line, vec![point]);
    }

    #[test]
    fn test_negative_coordinates() {
        let line: Vec<Point> =
            Bresenham::new(Point::new(-2, -3), Point::new(2, 3)).collect();
        assert_eq!(line[0], Point::new(-2, -3));
        assert_eq!(line[line.len() - 1], Point::new(2, 3));

        // Test points are connected with no gaps
        for i in 1..line.len() {
            let current = line[i];
            let previous = line[i - 1];
            let distance = ((current.x - previous.x).pow(2)
                + (current.y - previous.y).pow(2)) as f32;
            assert!(
                distance <= 2.0,
                "Points should be adjacent: {:?} and {:?}",
                previous,
                current
            );
        }
    }

    #[test]
    fn test_different_quadrants() {
        // Test lines in different quadrants of the coordinate system
        let quadrants = [
            (Point::new(0, 0), Point::new(5, 5)),   // Q1
            (Point::new(0, 0), Point::new(-5, 5)),  // Q2
            (Point::new(0, 0), Point::new(-5, -5)), // Q3
            (Point::new(0, 0), Point::new(5, -5)),  // Q4
        ];

        for (start, end) in quadrants.iter() {
            let line: Vec<Point> = Bresenham::new(*start, *end).collect();
            assert_eq!(line[0], *start);
            assert_eq!(line[line.len() - 1], *end);

            // Check connectivity
            for i in 1..line.len() {
                let a = line[i - 1];
                let b = line[i];
                let dx = (b.x - a.x).abs();
                let dy = (b.y - a.y).abs();
                assert!(dx <= 1 && dy <= 1, "Points should be adjacent");
            }
        }
    }
}
