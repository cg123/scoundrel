pub use scoundrel_util as util;
pub use scoundrel_geometry as geometry;
pub use scoundrel_algorithm as algorithm;
#[cfg(feature = "ui")]
pub use scoundrel_ui as ui;

use wgpu::Instance;

pub struct Engine<S> {
    pub gpu: Instance,
    pub state: S,
}

impl<S> Engine<S> {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
