pub use scoundrel_algorithm as algorithm;
pub use scoundrel_geometry as geometry;
#[cfg(feature = "ui")]
pub use scoundrel_ui as ui;
pub use scoundrel_util as util;

use thiserror::Error;
use wgpu::{Backends, Device, Instance, Queue, RequestDeviceError};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("No compatible adapter")]
    NoAdapter,
    #[error("Error requesting WGPU device")]
    RequestDeviceError(wgpu::RequestDeviceError),
}

impl From<wgpu::RequestDeviceError> for EngineError {
    fn from(err: RequestDeviceError) -> Self {
        EngineError::RequestDeviceError(err)
    }
}

pub struct GpuState {
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
}

impl GpuState {
    pub async fn initialize() -> Result<GpuState, EngineError> {
        let instance = Instance::new(Backends::PRIMARY);
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
        {
            Some(adapter) => adapter,
            None => return Err(EngineError::NoAdapter),
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

        Ok(GpuState {
            instance,
            device,
            queue,
        })
    }
}

pub struct Engine<S: GameState> {
    pub state: S,
    pub gpu: GpuState,
}

pub enum TickResult {
    Continue,
    Exit,
}

pub trait GameState {
    type Error: std::error::Error;
    fn initialize(&mut self, _gpu: &mut GpuState) -> Result<(), Self::Error> {
        Ok(())
    }
    fn tick(&mut self, _gpu: &mut GpuState) -> Result<TickResult, Self::Error> {
        Ok(TickResult::Exit)
    }
}

impl<S: GameState> Engine<S> {
    pub async fn new(state: S) -> Result<Engine<S>, EngineError> {
        Ok(Engine {
            state,
            gpu: GpuState::initialize().await?,
        })
    }

    pub fn run(mut self) -> Result<(), S::Error> {
        loop {
            match self.state.tick(&mut self.gpu) {
                Ok(TickResult::Continue) => {}
                Ok(TickResult::Exit) => {
                    break;
                }
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::GpuState;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn gpu() {
        pollster::block_on(async {
            let state = GpuState::initialize().await.unwrap();
            println!("GPU device: {:?}", state.device)
        });
    }
}
