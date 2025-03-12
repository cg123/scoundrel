pub use bounds::Bounds;
pub use matrix::Mat2;
pub use neighborhood::MooreNeighbor;
pub use vector::{Axis2D, Vector2, Vector3, Vector4};

mod bounds;
pub mod bsp;
mod grid2d;
mod half_space;
mod matrix;
mod neighborhood;
pub mod quadtree;
pub mod tilebin;
pub mod vector;

/// A 2D point with integer coordinates.
///
/// Alias for `Vector2<i32>`.
pub type Point = Vector2<i32>;

/// A 3D point with integer coordinates.
///
/// Alias for `Vector3<i32>`.
pub type Point3 = Vector3<i32>;

/// A rectangle in 2D space with integer coordinates.
///
/// Alias for `Bounds<i32>`.
pub type Rect = Bounds<i32>;
/// Grid data structure and related iterators.
pub use grid2d::{Grid2D, GridCoordIterator, GridIterator, GridNeighborhoodIterator};
/// Half space and orthogonal line primitives for spatial partitioning.
pub use half_space::{AxialHalfSpace, OrthoLine};
/// Spatial lookup data structure for tile-based games.
pub use tilebin::TileBin;
