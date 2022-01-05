use crate::Point;

pub struct Bresenham {
    delta: Point,
    step: Point,
    error: i32,

    current: Point,
    end: Point,
}

impl Iterator for Bresenham {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let point = self.current;

        let e2 = self.error * 2;
        if e2 >= self.delta.y {
            self.error += self.delta.y;
            self.current.x += self.step.x;
        }
        if e2 <= self.delta.x {
            self.error += self.delta.x;
            self.current.y += self.step.y;
        }

        Some(point)
    }
}

impl Bresenham {
    pub fn new(pt0: Point, pt1: Point) -> Bresenham {
        let delta = Point::new((pt1.x - pt0.x).abs(), -(pt1.y - pt0.y).abs());
        let step = Point::new(
            if pt0.x < pt1.x { 1 } else { -1 },
            if pt0.y < pt1.y { 1 } else { -1 },
        );
        Bresenham {
            delta,
            step,
            error: delta.x + delta.y,
            current: pt0,
            end: pt1,
        }
    }
}
