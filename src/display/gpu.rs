use pollster::FutureExt;
use thiserror::Error;
use wgpu::{
    Adapter, CommandEncoderDescriptor, Device, Instance, Queue, RequestAdapterError,
    RequestDeviceError, Surface, SurfaceError, TextureView, TextureViewDescriptor,
};

use super::window::WindowContext;

/// Shared GPU resources.
pub struct GpuContext {
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

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to request next texture: {0}")]
    GetFrame(#[from] SurfaceError),
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
            adapter,
            device,
            queue,
        })
    }

    /// Acquires a frame, runs the render function, and presents.
    pub fn render_frame<F>(&self, target: &WindowContext, f: F) -> Result<(), RenderError>
    where
        F: FnOnce(&mut wgpu::CommandEncoder, &TextureView),
    {
        let frame = target.surface.get_current_texture()?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        f(&mut encoder, &view);

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        target.window.request_redraw();
        Ok(())
    }
}
