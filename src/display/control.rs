mod frame;
mod gui;

pub use gui::GuiContext;

use egui::Context;
use wgpu::{
    Color, Device, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp,
};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::WindowId;

use super::gpu::GpuContext;
use super::window::WindowContext;
use super::{RenderError, render_frame};

/// Control window with egui GUI.
pub struct ControlWindow {
    window: WindowContext,
    gui: GuiContext,
}

impl ControlWindow {
    pub fn new(window: WindowContext, gui: GuiContext) -> Self {
        Self { window, gui }
    }

    pub fn window_id(&self) -> WindowId {
        self.window.window.id()
    }

    pub fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        self.window.resize(device, size);
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        self.gui.handle_event(&self.window.window, event)
    }

    pub fn render<F>(&mut self, gpu: &GpuContext, ui_fn: F) -> Result<(), RenderError>
    where
        F: FnMut(&Context),
    {
        let frame = self
            .gui
            .prepare_frame(&self.window.window, &gpu.device, &gpu.queue, ui_fn);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.window.config.width, self.window.config.height],
            pixels_per_point: frame.pixels_per_point,
        };

        let device = &gpu.device;
        let queue = &gpu.queue;

        render_frame(gpu, &self.window, |encoder, view| {
            frame
                .renderer
                .update_buffers(device, queue, encoder, &frame.tris, &screen_descriptor);

            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass = render_pass.forget_lifetime();
            frame
                .renderer
                .render(&mut render_pass, &frame.tris, &screen_descriptor);
        })
    }
}
