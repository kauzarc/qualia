use super::super::context::{GpuContext, WindowContext};

use super::{RenderError, render_frame};

pub struct ViewRenderer;

impl ViewRenderer {
    pub fn render(&mut self, gpu: &GpuContext, target: &WindowContext) -> Result<(), RenderError> {
        render_frame(gpu, target, |_encoder, _view| {
            // TODO: record render pass for custom shaders/triangles
        })
    }
}
