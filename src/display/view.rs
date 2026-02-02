use wgpu::{
    Color, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
};
use winit::dpi::PhysicalSize;

use super::gpu::GpuContext;
use super::window::WindowContext;
use super::{RenderError, render_frame};
use crate::inference::VisualParams;

/// View window for visual output.
pub struct ViewWindow {
    pub window: WindowContext,
}

impl ViewWindow {
    pub fn new(window: WindowContext) -> Self {
        Self { window }
    }

    pub fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        self.window.resize(device, size);
    }

    pub fn render(&self, gpu: &GpuContext, params: &VisualParams) -> Result<(), RenderError> {
        let r = params.actions[0].get();
        let g = params.actions[1].get();
        let b = params.actions[2].get();

        render_frame(gpu, &self.window, |encoder, view| {
            encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("View Clear"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color { r, g, b, a: 1.0 }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        })
    }
}
