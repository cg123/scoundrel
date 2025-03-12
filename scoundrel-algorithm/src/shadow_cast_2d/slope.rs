use std::cmp::Ordering;

/// Represents a slope as a rise and run.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Slope {
    pub rise: i32,
    pub run: i32,
}

impl Slope {
    pub const ONE: Slope = Slope::new(1, 1);
    pub const ZERO: Slope = Slope::new(0, 0);

    pub const fn new(mut rise: i32, mut run: i32) -> Slope {
        if run < 0 {
            rise *= -1;
            run *= -1;
        }
        Slope { rise, run }
    }
}

impl PartialOrd<Self> for Slope {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Slope {
    fn cmp(&self, other: &Self) -> Ordering {
        // sy / sx <>= oy / ox
        // sy * ox / sx <>= oy
        // sy * ox <>= oy * sx
        // *given* our invariant that sx, ox > 0
        (self.rise * other.run).cmp(&(other.rise * self.run))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slope_comparison() {
        // Test slope ordering
        let s1 = Slope::new(1, 2); // 0.5
        let s2 = Slope::new(1, 1); // 1.0
        let s3 = Slope::new(2, 1); // 2.0
        let s4 = Slope::new(-1, -2); // 0.5 (normalized to 1/2)

        assert!(s1 < s2);
        assert!(s2 < s3);
        assert_eq!(s1, s4); // Same slope after normalization

        // Test negative run normalization
        let s5 = Slope::new(1, -2);
        let s6 = Slope::new(-1, 2);
        assert_eq!(s5, s6);
    }

    #[test]
    fn test_slope_constants() {
        assert_eq!(Slope::ZERO, Slope::new(0, 0));
        assert_eq!(Slope::ONE, Slope::new(1, 1));
    }
}
