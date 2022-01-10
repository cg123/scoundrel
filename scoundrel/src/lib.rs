#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "terminal")]
mod terminal;

#[cfg(feature = "gpu")]
pub use scoundrel_procedural::wgsl_module;
#[cfg(feature = "terminal")]
pub use terminal::TerminalState;

pub use scoundrel_algorithm as algorithm;
pub use scoundrel_geometry as geometry;
pub use scoundrel_util as util;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[cfg(feature = "gpu")]
    #[error("GPU error")]
    GpuError(#[from] gpu::GpuError),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

pub struct EngineState {
    #[cfg(feature = "gpu")]
    pub gpu: gpu::GpuState,
}

pub struct Engine<S: GameState> {
    pub game_state: S,
    pub state: EngineState,
}

pub enum TickResult {
    Continue,
    Exit,
}

pub trait GameState {
    type Error: std::error::Error;
    #[allow(unused)]
    fn initialize(&mut self, engine: &mut EngineState) -> Result<(), Self::Error> {
        Ok(())
    }
    #[allow(unused)]
    fn tick(&mut self, engine: &mut EngineState) -> Result<TickResult, Self::Error> {
        Ok(TickResult::Exit)
    }
}

impl<S: GameState> Engine<S> {
    pub fn new(game_state: S) -> Result<Engine<S>, EngineError> {
        let state = EngineState {
            #[cfg(feature = "gpu")]
            gpu: gpu::GpuState::new_sync()?,
        };
        Ok(Engine { game_state, state })
    }

    pub fn run(mut self) -> Result<(), S::Error> {
        self.game_state.initialize(&mut self.state)?;
        loop {
            match self.game_state.tick(&mut self.state)? {
                TickResult::Exit => break,
                TickResult::Continue => continue,
            }
        }
        Ok(())
    }
}
