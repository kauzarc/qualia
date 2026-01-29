use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use wgpu::{Device, TextureFormat};
use winit::window::Window;

/// State required to render the GUI.
pub struct GuiContext {
    pub context: Context,
    pub state: State,
    pub renderer: Renderer,
}

impl GuiContext {
    pub fn new(window: &Window, device: &Device, output_format: TextureFormat) -> Self {
        let context = Context::default();

        let state = State::new(
            context.clone(),
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            None,
        );

        let renderer = Renderer::new(
            device,
            output_format,
            RendererOptions {
                msaa_samples: 1,
                ..Default::default()
            },
        );

        Self {
            context,
            state,
            renderer,
        }
    }
}
