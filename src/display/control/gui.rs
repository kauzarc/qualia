use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use wgpu::{Device, Queue, TextureFormat};
use winit::event::WindowEvent;
use winit::window::Window;

use super::frame::PreparedFrame;

/// State required to render the GUI.
pub struct GuiContext {
    context: Context,
    state: State,
    renderer: Renderer,
}

impl GuiContext {
    #[allow(clippy::cast_possible_truncation)]
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

    /// Runs an egui frame: collects input, executes UI, tessellates, and uploads textures.
    ///
    /// Returns a `PreparedFrame` that holds the renderer and frees textures when dropped.
    pub fn prepare_frame<F>(
        &mut self,
        window: &Window,
        device: &Device,
        queue: &Queue,
        ui_fn: F,
    ) -> PreparedFrame<'_>
    where
        F: FnMut(&Context),
    {
        let raw_input = self.state.take_egui_input(window);
        let full_output = self.context.run(raw_input, ui_fn);

        self.state
            .handle_platform_output(window, full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        PreparedFrame::new(
            &mut self.renderer,
            device,
            queue,
            tris,
            full_output.pixels_per_point,
            full_output.textures_delta,
        )
    }

    /// Handles a window event, returning true if egui consumed it.
    pub fn handle_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }
}
