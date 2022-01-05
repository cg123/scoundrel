use anyhow::Result;
use serde::{Deserialize, Serialize};
use tui::style::Color;

pub use layout::*;
pub use menu::Menu;

use scoundrel_geometry::Rect;

mod layout;
mod menu;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Rgb8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb8 {
    pub fn new<T: Into<u8>>(r: T, g: T, b: T) -> Rgb8 {
        Rgb8 {
            r: r.into(),
            g: g.into(),
            b: b.into(),
        }
    }

    pub fn black() -> Rgb8 {
        Rgb8 { r: 0, g: 0, b: 0 }
    }

    pub fn grey<T: Into<u8>>(level: T) -> Rgb8 {
        let b = level.into();
        Rgb8::new(b, b, b)
    }

    pub fn to_grey(self) -> Rgb8 {
        let weighted = 0.30 * self.r as f32 + 0.59 * self.g as f32 + 0.11 * self.b as f32;
        Rgb8::grey(weighted as u8)
    }
}

impl From<Rgb8> for Color {
    fn from(rgb8: Rgb8) -> Color {
        Color::Rgb(rgb8.r, rgb8.g, rgb8.b)
    }
}

pub trait Element {
    type Data: Copy + 'static;
    fn layout(&self, rect: Rect) -> Result<LayoutElement<Self::Data>>;
    fn render_part<B: tui::backend::Backend>(
        &mut self,
        label: Self::Data,
        rect: Rect,
        f: &mut tui::Frame<B>,
    ) -> Result<()>;
    fn render<B: tui::backend::Backend>(&mut self, rect: Rect, f: &mut tui::Frame<B>) {
        self.layout(rect)
            .unwrap()
            .visit_data(|data: &Self::Data, elem: &LayoutElement<Self::Data>| {
                self.render_part(*data, elem.rect, f)
                    .expect("Failed to render part!");
            })
            .unwrap();
    }
}
