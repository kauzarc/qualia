use wgpu::{Color, CommandEncoder, LoadOp, Operations, RenderPassDescriptor, StoreOp, TextureView};

use super::super::context::{GpuContext, WindowContext};
use super::{RenderError, render_frame};
use crate::inference::VisualParams;

pub struct ViewRenderer;

impl ViewRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &mut self,
        gpu: &GpuContext,
        target: &WindowContext,
        params: &VisualParams,
    ) -> Result<(), RenderError> {
        // Use first 3 params as RGB
        let r = params.actions[0].get() as f64;
        let g = params.actions[1].get() as f64;
        let b = params.actions[2].get() as f64;

        render_frame(gpu, target, |encoder, view| {
            Self::clear_screen(encoder, view, Color { r, g, b, a: 1.0 });
        })
    }

    fn clear_screen(encoder: &mut CommandEncoder, view: &TextureView, color: Color) {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("View Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(color),
                    store: StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
}
