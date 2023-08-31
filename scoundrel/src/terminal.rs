use crate::geometry::{Point, Rect};
use crossterm::{
    event,
    event::{Event, KeyEvent, KeyEventKind},
    terminal::enable_raw_mode,
};
use std::io;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Frame, Terminal};

pub struct TerminalState {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub pressed: Option<KeyEvent>,
}
impl TerminalState {
    pub fn new() -> io::Result<TerminalState> {
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        Ok(TerminalState {
            terminal,
            pressed: None,
        })
    }

    pub fn update_keyboard(&mut self, timeout: Duration) -> io::Result<()> {
        if event::poll(timeout)? {
            self.pressed = match event::read() {
                Ok(Event::Key(event)) if event.kind == KeyEventKind::Press => Some(event),
                _ => None,
            };
        } else {
            self.pressed = None;
        }
        Ok(())
    }

    pub fn size(&self) -> io::Result<Point> {
        let r: Rect = self.terminal.size()?.into();
        Ok(r.size())
    }

    pub fn draw<F: FnOnce(&mut Frame<CrosstermBackend<io::Stdout>>)>(&mut self, f: F) {
        self.terminal.draw(f).expect("Failed to draw!");
    }
}
