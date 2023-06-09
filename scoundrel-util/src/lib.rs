mod ascii_glyph;
mod intern_pool;
mod memoizer;
pub mod numeric;
mod pointer_equality;
mod pq_entry;

pub use ascii_glyph::AsciiGlyph;
pub use intern_pool::{InternID, InternPool};
pub use memoizer::Memoizer;
pub use numeric::NonNaN32;
pub use pointer_equality::PointerEqual;
pub use pq_entry::PQEntry;

#[macro_export]
macro_rules! ignore_ident {
    ($id:ident, $($tail:tt)*) => { $($tail)* };
}
