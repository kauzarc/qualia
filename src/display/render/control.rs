use egui::CentralPanel;
use wgpu::{LoadOp, Operations, RenderPassColorAttachment, StoreOp};

use super::super::context::{ControlWindow, GpuContext, PreparedFrame};
use super::{RenderError, render_frame};

pub struct ControlRenderer;

impl ControlRenderer {
    #[allow(clippy::unused_self)]
    pub fn render(
        &mut self,
        gpu: &GpuContext,
        control: &mut ControlWindow,
    ) -> Result<(), RenderError> {
        let frame =
            control
                .gui
                .prepare_frame(&control.window.window, &gpu.device, &gpu.queue, |ctx| {
                    CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Hello, World!");
                    });
                });

        self.render_frame(gpu, &control.window, frame)
        // frame dropped here, textures freed automatically
    }

    fn render_frame(
        &self,
        gpu: &GpuContext,
        window: &super::super::context::WindowContext,
        frame: PreparedFrame<'_>,
    ) -> Result<(), RenderError> {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [window.config.width, window.config.height],
            pixels_per_point: frame.pixels_per_point,
        };

        let device = &gpu.device;
        let queue = &gpu.queue;

        render_frame(gpu, window, |encoder, view| {
            frame
                .renderer
                .update_buffers(device, queue, encoder, &frame.tris, &screen_descriptor);

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::BLACK),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: render_pass is dropped before encoder.finish() is called
            let mut render_pass = render_pass.forget_lifetime();
            frame
                .renderer
                .render(&mut render_pass, &frame.tris, &screen_descriptor);
        })
    }
}
