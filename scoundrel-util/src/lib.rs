mod ascii_glyph;
pub mod numeric;
mod pq_entry;

pub use ascii_glyph::AsciiGlyph;
pub use numeric::NonNaN32;
pub use pq_entry::PQEntry;

#[macro_export]
macro_rules! ignore_ident {
    ($id:ident, $($tail:tt)*) => { $($tail)* };
}
