use thiserror::Error;
use wgpu::SurfaceError;

mod control;
mod view;

pub use control::ControlRenderer;
pub use view::ViewRenderer;

use crate::context::{GpuContext, WindowContext};

pub trait Renderer {
    fn render(&mut self, gpu: &GpuContext, target: &WindowContext) -> Result<(), RenderError> {
        let frame = target.surface.get_current_texture()?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = gpu.device.create_command_encoder(&Default::default());

        todo!();

        gpu.queue.submit(Some(encoder.finish()));
        frame.present();

        target.window.request_redraw();
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to request next texture: {0}")]
    GetFrame(#[from] SurfaceError),
}
