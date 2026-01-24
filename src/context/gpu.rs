use pollster::FutureExt;
use thiserror::Error;
use wgpu::{Adapter, Device, Instance, Queue, RequestAdapterError, RequestDeviceError, Surface};

/// Shared GPU resources.
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

#[derive(Error, Debug)]
pub enum GpuContextError {
    #[error("wgpu::Adapter request failed: {0}")]
    RequestAdapter(#[from] RequestAdapterError),

    #[error("wgpu::Device request failed: {0}")]
    RequestDevice(#[from] RequestDeviceError),
}

impl GpuContext {
    pub fn try_new(
        instance: &Instance,
        compatible_surface: &Surface,
    ) -> Result<Self, GpuContextError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(compatible_surface),
                force_fallback_adapter: false,
            })
            .block_on()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Qualia Device"),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            })
            .block_on()?;

        Ok(Self {
            instance: instance.clone(),
            adapter,
            device,
            queue,
        })
    }
}
