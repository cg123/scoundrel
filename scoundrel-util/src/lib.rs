mod ascii_glyph;
mod min_heap_entry;
pub mod numeric;

pub use ascii_glyph::AsciiGlyph;
pub use min_heap_entry::MinHeapEntry;
pub use numeric::NonNaN32;

/// Macro that ignores the first identifier and returns the tail of the pattern.
///
/// This utility macro is used in macro metaprogramming to discard an identifier
/// while keeping the rest of the pattern. It's particularly useful in macros that
/// need to work with identifiers but only care about their presence, not their value.
///
/// # Examples
///
/// ```
/// use scoundrel_util::ignore_ident;
///
/// // Returns 42, ignoring the identifier 'foo'
/// let value = ignore_ident!(foo, 42);
/// assert_eq!(value, 42);
/// ```
#[macro_export]
macro_rules! ignore_ident {
    ($id:ident, $($tail:tt)*) => { $($tail)* };
}
