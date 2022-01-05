pub use bounds::Bounds;
pub use bresenham::Bresenham;
pub use matrix::Mat2;
pub use neighborhood::Neighbor;
pub use vector::{Axis2D, Vector2};

mod bounds;
mod bresenham;
pub mod grid2d;
mod matrix;
mod neighborhood;
pub mod vector;

pub type Point = Vector2<i32>;
pub type Rect = Bounds<i32>;
