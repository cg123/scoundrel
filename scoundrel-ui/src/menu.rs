use tui::style::*;
use tui::widgets::{List, ListItem, Widget};

pub struct Menu<T> {
    pub choices: Vec<(T, String)>,
    pub selected_idx: usize,

    pub style: Style,
    pub selected_style: Style,
}

impl<T> Menu<T> {
    pub fn select_prev(&mut self) {
        self.selected_idx = (self.selected_idx + self.choices.len() - 1) % self.choices.len();
    }
    pub fn select_next(&mut self) {
        self.selected_idx = (self.selected_idx + 1) % self.choices.len();
    }
    pub fn selected(&self) -> Option<&T> {
        if self.selected_idx < self.choices.len() {
            Some(&self.choices[self.selected_idx].0)
        } else {
            None
        }
    }

    pub fn widget(&self) -> impl Widget {
        List::new(
            self.choices
                .iter()
                .enumerate()
                .map(|(idx, (_, label))| {
                    let style = if idx == self.selected_idx {
                        self.selected_style
                    } else {
                        self.style
                    };
                    ListItem::new(label.to_owned()).style(style)
                })
                .collect::<Vec<_>>(),
        )
    }
}

impl<T> Menu<T> {
    pub fn new() -> Menu<T> {
        Menu {
            choices: vec![],
            selected_idx: 0,
            style: Default::default(),
            selected_style: Default::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn selected_style(mut self, style: Style) -> Self {
        self.selected_style = style;
        self
    }
    pub fn choice<P: ToString>(mut self, id: T, label: P) -> Self {
        self.choices.push((id, label.to_string()));
        self
    }
}

impl<T> Default for Menu<T> {
    fn default() -> Self {
        Self::new()
    }
}
