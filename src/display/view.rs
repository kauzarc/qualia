use wgpu::{
    Adapter, Color, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp,
};
use winit::dpi::PhysicalSize;

use super::gpu::{GpuContext, RenderError};
use super::window::{UnconfiguredWindow, WindowContext};
use crate::inference::ControlVoltage;

/// View window for visual output.
pub struct ViewWindow {
    pub window: WindowContext,
}

impl ViewWindow {
    pub fn new(window: UnconfiguredWindow, adapter: &Adapter, device: &Device) -> Self {
        Self {
            window: window.configure(adapter, device),
        }
    }

    pub fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        self.window.resize(device, size);
    }

    pub fn render(&self, gpu: &GpuContext, actions: &[ControlVoltage]) -> Result<(), RenderError> {
        let r = actions.first().copied().map_or(0.0, ControlVoltage::get);
        let g = actions.get(1).copied().map_or(0.0, ControlVoltage::get);
        let b = actions.get(2).copied().map_or(0.0, ControlVoltage::get);

        gpu.render_frame(&self.window, |encoder, view| {
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
