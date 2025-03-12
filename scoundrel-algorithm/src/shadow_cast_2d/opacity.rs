/// Represents whether a tile or object blocks light for field of view calculations.
///
/// This enum is used by the shadowcasting algorithm to determine if light
/// can pass through a tile when calculating field of view.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Opacity {
    /// The tile blocks light completely (walls, solid objects).
    Opaque,
    /// The tile allows light to pass through (empty space, windows).
    Transparent,
}
