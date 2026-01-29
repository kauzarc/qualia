use crate::context::{GpuContext, WindowContext};

use super::{render_frame, RenderError};

pub struct ViewRenderer;

impl ViewRenderer {
    pub fn render(&mut self, gpu: &GpuContext, target: &WindowContext) -> Result<(), RenderError> {
        render_frame(gpu, target, |_encoder, _view| {
            // TODO: record render pass for custom shaders/triangles
        })
    }
}
