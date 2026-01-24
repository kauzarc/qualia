use std::sync::Arc;

use thiserror::Error;
use wgpu::{Adapter, CreateSurfaceError, Device, Surface, SurfaceConfiguration};
use winit::{dpi::PhysicalSize, error::OsError, window::Window};

/// A renderable surface associated with a specific OS window.
pub struct WindowContext {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
}

#[derive(Error, Debug)]
pub enum WindowContextError {
    #[error("can't create winit::Window: {0}")]
    CreateWindow(#[from] OsError),

    #[error("can't create wgpu::Surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),
}

impl WindowContext {
    pub fn from_raw(
        window: Arc<Window>,
        surface: Surface<'static>,
        adapter: &Adapter,
        device: &Device,
    ) -> Self {
        let size = window.inner_size();
        let caps = surface.get_capabilities(adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Self {
            window,
            surface,
            config,
        }
    }

    pub fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(device, &self.config);
        }
    }
}
