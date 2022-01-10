use thiserror::Error;
use wgpu::{Adapter, Backends, Device, Instance, Queue, RequestDeviceError};

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("No compatible adapter")]
    NoAdapter,
    #[error("Error requesting WGPU device")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
}

pub struct GpuState {
    pub adapter: Adapter,
    pub instance: Instance,
    pub device: Device,
    pub queue: Queue,
}

impl GpuState {
    pub async fn initialize() -> Result<GpuState, GpuError> {
        let instance = Instance::new(Backends::PRIMARY);
        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
        {
            Some(adapter) => adapter,
            None => return Err(GpuError::NoAdapter),
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty()
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

        Ok(GpuState {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn new_sync() -> Result<GpuState, GpuError> {
        pollster::block_on(GpuState::initialize())
    }
}
