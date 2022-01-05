use super::Point;

macro_rules! int_enum {
    ($vis:vis enum $name:ident {
        $(
            $mem:ident = $val:literal
        ),+ $(,)?
    }) => {
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
    pub enum Neighbor {
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

impl Neighbor {
    pub fn for_each<F: FnMut(Neighbor)>(mut f: F) {
        for idx in 0..8 {
            f(Self::from_index(idx).unwrap())
        }
    }
    pub fn all() -> Vec<Neighbor> {
        (0..8).map(|idx| Self::from_index(idx).unwrap()).collect()
    }

    pub fn opposite(&self) -> Neighbor {
        Self::from_index((self.to_index() + 4) % 8).unwrap()
    }

    pub fn offset(&self) -> Point {
        let (dx, dy) = match self {
            Neighbor::LeftUp => (-1, -1),
            Neighbor::Up => (0, -1),
            Neighbor::RightUp => (1, -1),
            Neighbor::Right => (1, 0),
            Neighbor::RightDown => (1, 1),
            Neighbor::Down => (0, 1),
            Neighbor::LeftDown => (-1, 1),
            Neighbor::Left => (-1, 0),
        };
        Point::new(dx, dy)
    }

    pub fn offset_magnitude(&self) -> f32 {
        match self {
            Neighbor::Up => 1.0,
            Neighbor::Right => 1.0,
            Neighbor::Down => 1.0,
            Neighbor::Left => 1.0,
            _ => std::f32::consts::SQRT_2,
        }
    }

    // index of the neighbor in a row-major 3x3 window
    pub fn window_index(&self) -> usize {
        match self {
            Neighbor::LeftUp => 0,
            Neighbor::Up => 1,
            Neighbor::RightUp => 2,
            Neighbor::Left => 3,
            // no Neighbor::Center
            Neighbor::Right => 5,
            Neighbor::LeftDown => 6,
            Neighbor::Down => 7,
            Neighbor::RightDown => 8,
        }
    }

    pub fn from_window_index(index: usize) -> Option<Neighbor> {
        Some(match index {
            0 => Neighbor::LeftUp,
            1 => Neighbor::Up,
            2 => Neighbor::RightUp,
            3 => Neighbor::Left,
            4 => return None,
            5 => Neighbor::Right,
            6 => Neighbor::LeftDown,
            7 => Neighbor::Down,
            8 => Neighbor::RightDown,
            _ => return None,
        })
    }
}
