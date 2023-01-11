#[cfg(feature = "terminal")]
mod terminal;

#[cfg(feature = "terminal")]
pub use terminal::TerminalState;

pub use scoundrel_algorithm as algorithm;
pub use scoundrel_geometry as geometry;
pub use scoundrel_util as util;
