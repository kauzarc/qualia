use egui::{ClippedPrimitive, TextureId, TexturesDelta};
use egui_wgpu::Renderer;
use wgpu::{Device, Queue};

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
}

impl Drop for PreparedFrame<'_> {
    fn drop(&mut self) {
        for id in &self.textures_to_free {
            self.renderer.free_texture(id);
        }
    }
}
