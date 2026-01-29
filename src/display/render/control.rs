use super::super::context::{ControlWindow, GpuContext};

use super::{RenderError, render_frame};

pub struct ControlRenderer;

impl ControlRenderer {
    #[allow(clippy::unused_self)]
    pub fn render(
        &mut self,
        gpu: &GpuContext,
        control: &mut ControlWindow,
    ) -> Result<(), RenderError> {
        render_frame(gpu, &control.window, |_encoder, _view| {
            // TODO: render egui
        })
    }
}
