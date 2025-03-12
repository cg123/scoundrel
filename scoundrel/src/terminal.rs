use crate::geometry::Point;
use crossterm::{
    event,
    event::{Event, KeyEvent, KeyEventKind},
    terminal::enable_raw_mode,
};
use std::io;
use std::time::Duration;
use tui::{backend::CrosstermBackend, Frame, Terminal};

/// Manages the terminal UI state and keyboard input processing.
///
/// Thin wrapper around the `Terminal` struct from the `tui` crate, with additional
/// functionality for handling keyboard input.
pub struct TerminalState {
    /// The ratatui terminal instance used for rendering.
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,

    /// The most recently pressed key, if any.
    pub pressed: Option<KeyEvent>,
}
impl TerminalState {
    /// Creates a new terminal state with raw mode enabled and the terminal cleared.
    ///
    /// # Returns
    /// A new `TerminalState` instance wrapped in `io::Result`.
    ///
    /// # Errors
    /// Returns an error if terminal initialization fails or raw mode cannot be enabled.
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

    /// Checks for keyboard events and updates the `pressed` field.
    ///
    /// This method polls for keyboard events using the specified timeout. If a key press
    /// is detected, it updates the `pressed` field with the key event. Otherwise, it
    /// clears the `pressed` field.
    ///
    /// # Arguments
    /// * `timeout` - The maximum duration to wait for a keyboard event.
    ///
    /// # Returns
    /// `Ok(())` if the operation succeeds, or an IO error otherwise.
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

    /// Gets the current terminal size as a geometric Point.
    ///
    /// Retrieves the dimensions of the terminal window and converts them to
    /// a Point structure, where x is the width and y is the height.
    ///
    /// # Returns
    /// A `Point` with the terminal width as `x` and height as `y`.
    ///
    /// # Errors
    /// Returns an error if getting the terminal size fails.
    pub fn size(&self) -> io::Result<Point> {
        let sz: tui::prelude::Size = self.terminal.size()?;
        Ok(sz.into())
    }

    /// Draws to the terminal using the provided rendering function.
    ///
    /// This method handles the actual rendering process by passing a frame to the
    /// provided function and then rendering that frame to the terminal.
    ///
    /// # Arguments
    /// * `f` - A function that takes a mutable reference to a `Frame` and renders to it.
    ///
    /// # Panics
    /// Panics if the drawing operation fails.
    pub fn draw<F: FnOnce(&mut Frame)>(&mut self, f: F) {
        self.terminal.draw(f).expect("Failed to draw!");
    }
}
