use egui::{ClippedPrimitive, TextureId, TexturesDelta};
use egui_wgpu::Renderer;
use wgpu::{
    Color, Device, LoadOp, Operations, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    StoreOp, TextureView,
};

/// A prepared egui frame ready for rendering.
///
/// Uploads textures on creation and frees them on drop.
pub struct PreparedFrame<'a> {
    pub renderer: &'a mut Renderer,
    pub tris: Vec<ClippedPrimitive>,
    pub pixels_per_point: f32,
    textures_to_free: Vec<TextureId>,
}

impl<'a> PreparedFrame<'a> {
    pub(super) fn new(
        renderer: &'a mut Renderer,
        device: &Device,
        queue: &Queue,
        tris: Vec<ClippedPrimitive>,
        pixels_per_point: f32,
        textures_delta: TexturesDelta,
    ) -> Self {
        for (id, image_delta) in textures_delta.set {
            renderer.update_texture(device, queue, id, &image_delta);
        }

        Self {
            renderer,
            tris,
            pixels_per_point,
            textures_to_free: textures_delta.free,
        }
    }

    pub fn render(
        self,
        device: &Device,
        queue: &Queue,
        encoder: &mut wgpu::CommandEncoder,
        view: &TextureView,
        size: [u32; 2],
    ) {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: self.pixels_per_point,
        };

        self.renderer
            .update_buffers(device, queue, encoder, &self.tris, &screen_descriptor);

        let mut render_pass = Self::begin_render_pass(encoder, view);
        self.renderer
            .render(&mut render_pass, &self.tris, &screen_descriptor);
    }

    fn begin_render_pass(
        encoder: &mut wgpu::CommandEncoder,
        view: &TextureView,
    ) -> wgpu::RenderPass<'static> {
        encoder
            .begin_render_pass(&RenderPassDescriptor {
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
            })
            .forget_lifetime()
    }
}

impl Drop for PreparedFrame<'_> {
    fn drop(&mut self) {
        for id in &self.textures_to_free {
            self.renderer.free_texture(id);
        }
    }
}
