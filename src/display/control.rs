mod frame;
mod gui;
mod panel;

pub use gui::GuiContext;

use panel::ControlPanel;
use wgpu::Device;
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::WindowId;

use super::gpu::{GpuContext, RenderError};
use super::window::WindowContext;

/// Control window with egui GUI.
pub struct ControlWindow {
    window: WindowContext,
    gui: GuiContext,
    panel: ControlPanel,
}

impl ControlWindow {
    pub fn new(window: WindowContext, gui: GuiContext, proxy: EventLoopProxy<AppEvent>) -> Self {
        Self {
            window,
            gui,
            panel: ControlPanel::new(proxy),
        }
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

    pub fn render(&mut self, gpu: &GpuContext) -> Result<(), RenderError> {
        let frame = self
            .gui
            .prepare_frame(&self.window.window, &gpu.device, &gpu.queue, |ctx| {
                self.panel.show(ctx);
            });
        let size = [self.window.config.width, self.window.config.height];

        gpu.render_frame(&self.window, |encoder, view| {
            frame.render(&gpu.device, &gpu.queue, encoder, view, size);
        })
    }
}
