use super::Point;

macro_rules! int_enum {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {
        $(
            $mem:ident = $val:literal
        ),+ $(,)?
    }) => {
        $(#[$outer])*
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
        $vis enum $name {
            $(
                $mem = $val,
            )+
        }
        impl $name {
            pub fn from_index(int: usize) -> Option<Self> {
                match int {
                    $(
                        $val => Some(Self::$mem),
                    )+
                    _ => None
                }
            }
            pub fn to_index(self) -> usize {
                self as usize
            }
        }
    };
}

int_enum! {
    /// An enumeration representing the 8 possible neighbors in a Moore neighborhood.
    pub enum MooreNeighbor {
        Up = 0,
        RightUp = 1,
        Right = 2,
        RightDown = 3,
        Down = 4,
        LeftDown = 5,
        Left = 6,
        LeftUp = 7,
    }
}

impl MooreNeighbor {
    /// Calls the provided closure `f` once for each `MooreNeighbor` variant, in the order they are defined.
    pub fn for_each<F: FnMut(MooreNeighbor)>(mut f: F) {
        for idx in 0..8 {
            f(Self::from_index(idx).unwrap())
        }
    }

    /// Returns a vector containing all `MooreNeighbor` variants, in the order they are defined.
    pub fn all() -> Vec<MooreNeighbor> {
        (0..8).map(|idx| Self::from_index(idx).unwrap()).collect()
    }

    /// Returns the `MooreNeighbor` variant that is opposite to this one.
    pub fn opposite(&self) -> MooreNeighbor {
        Self::from_index((self.to_index() + 4) % 8).unwrap()
    }

    /// Returns the offset from the origin to the neighboring point.
    pub fn offset(&self) -> Point {
        let (dx, dy) = match self {
            MooreNeighbor::LeftUp => (-1, -1),
            MooreNeighbor::Up => (0, -1),
            MooreNeighbor::RightUp => (1, -1),
            MooreNeighbor::Right => (1, 0),
            MooreNeighbor::RightDown => (1, 1),
            MooreNeighbor::Down => (0, 1),
            MooreNeighbor::LeftDown => (-1, 1),
            MooreNeighbor::Left => (-1, 0),
        };
        Point::new(dx, dy)
    }

    /// Returns the magnitude of the offset from the origin to the neighboring point in the corresponding direction.
    /// This is `sqrt(2)` for diagonal neighbors, and `1` for non-diagonal neighbors.
    pub fn offset_magnitude(&self) -> f32 {
        match self {
            MooreNeighbor::Up => 1.0,
            MooreNeighbor::Right => 1.0,
            MooreNeighbor::Down => 1.0,
            MooreNeighbor::Left => 1.0,
            _ => std::f32::consts::SQRT_2,
        }
    }

    /// Returns the index of this neighbor in a row-major 3x3 window.
    pub fn window_index(&self) -> usize {
        match self {
            MooreNeighbor::LeftUp => 0,
            MooreNeighbor::Up => 1,
            MooreNeighbor::RightUp => 2,
            MooreNeighbor::Left => 3,
            // no Neighbor::Center
            MooreNeighbor::Right => 5,
            MooreNeighbor::LeftDown => 6,
            MooreNeighbor::Down => 7,
            MooreNeighbor::RightDown => 8,
        }
    }

    /// Returns the `MooreNeighbor` variant corresponding to the given index in a row-major 3x3 window.
    /// Returns `None` if there is no appropriate variant.
    pub fn from_window_index(index: usize) -> Option<MooreNeighbor> {
        Some(match index {
            0 => MooreNeighbor::LeftUp,
            1 => MooreNeighbor::Up,
            2 => MooreNeighbor::RightUp,
            3 => MooreNeighbor::Left,
            4 => return None,
            5 => MooreNeighbor::Right,
            6 => MooreNeighbor::LeftDown,
            7 => MooreNeighbor::Down,
            8 => MooreNeighbor::RightDown,
            _ => return None,
        })
    }
}
