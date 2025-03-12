mod ascii_glyph;
mod min_heap_entry;
pub mod numeric;

pub use ascii_glyph::AsciiGlyph;
pub use min_heap_entry::MinHeapEntry;
pub use numeric::NonNaN32;

#[macro_export]
macro_rules! ignore_ident {
    ($id:ident, $($tail:tt)*) => { $($tail)* };
}
