use crate::context::{GpuContext, GuiContext, WindowContext};

use super::{render_frame, RenderError};

pub struct ControlRenderer;

impl ControlRenderer {
    pub fn render(
        &mut self,
        gpu: &GpuContext,
        target: &WindowContext,
        _gui: &mut GuiContext,
    ) -> Result<(), RenderError> {
        render_frame(gpu, target, |_encoder, _view| {
            // TODO: render egui
        })
    }
}
