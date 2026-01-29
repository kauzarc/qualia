use thiserror::Error;
use wgpu::{CommandEncoder, SurfaceError, TextureView};

mod control;
mod view;

pub use control::ControlRenderer;
pub use view::ViewRenderer;

use crate::context::{GpuContext, WindowContext};

pub fn render_frame<F>(gpu: &GpuContext, target: &WindowContext, f: F) -> Result<(), RenderError>
where
    F: FnOnce(&mut CommandEncoder, &TextureView),
{
    let frame = target.surface.get_current_texture()?;
    let view = frame.texture.create_view(&Default::default());
    let mut encoder = gpu.device.create_command_encoder(&Default::default());

    f(&mut encoder, &view);

    gpu.queue.submit(Some(encoder.finish()));
    frame.present();
    target.window.request_redraw();
    Ok(())
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to request next texture: {0}")]
    GetFrame(#[from] SurfaceError),
}
