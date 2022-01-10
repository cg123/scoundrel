mod a_star;
mod shadow_cast;
mod space;

extern crate scoundrel_geometry;

pub use a_star::a_star;
pub use shadow_cast::cast_light;
pub use space::{BaseMap, MapOf, MappableMap, Opacity, Passability};
