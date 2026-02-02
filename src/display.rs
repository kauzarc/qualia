//! Display and rendering module.
//!
//! Manages two windows: the main visual output and the control panel.

mod control;
mod gpu;
mod view;
mod window;

use std::sync::mpsc::Sender;

use egui::CentralPanel;
use rtrb::Consumer;
use thiserror::Error;
use tracing::debug;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Instance, InstanceDescriptor, SurfaceError,
    TextureView, TextureViewDescriptor,
};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use control::{ControlWindow, GuiContext};
pub use gpu::GpuContext;
use gpu::GpuContextError;
use view::ViewWindow;
use window::{UnconfiguredWindow, WindowContext, WindowError};

use crate::inference::VisualParams;
use crate::trainer::Feedback;

/// Display subsystem managing windows and GPU rendering.
pub struct Display {
    gpu: GpuContext,
    view: ViewWindow,
    control: ControlWindow,
    params_consumer: Consumer<VisualParams>,
    current_params: VisualParams,
    _feedback_sender: Sender<Feedback>,
}

/// Errors that can occur during display operations.
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Failed to init view window: {0}")]
    InitViewWindow(WindowError),

    #[error("Failed to init control window: {0}")]
    InitControlWindow(WindowError),

    #[error("Failed to init GPU: {0}")]
    InitGpu(#[from] GpuContextError),

    #[error("Error while rendering: {0}")]
    Render(#[from] RenderError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to request next texture: {0}")]
    GetFrame(#[from] SurfaceError),
}

/// Acquires a frame, runs the render function, and presents.
fn render_frame<F>(gpu: &GpuContext, target: &WindowContext, f: F) -> Result<(), RenderError>
where
    F: FnOnce(&mut CommandEncoder, &TextureView),
{
    let frame = target.surface.get_current_texture()?;
    let view = frame.texture.create_view(&TextureViewDescriptor::default());
    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());

    f(&mut encoder, &view);

    gpu.queue.submit(Some(encoder.finish()));
    frame.present();
    target.window.request_redraw();
    Ok(())
}

impl Display {
    pub fn try_new(
        event_loop: &ActiveEventLoop,
        params_consumer: Consumer<VisualParams>,
        feedback_sender: Sender<Feedback>,
    ) -> Result<Self, DisplayError> {
        let instance = Instance::new(&InstanceDescriptor::default());

        debug!("Creating windows...");
        let view = UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Vision")
            .map_err(DisplayError::InitViewWindow)?;
        let control = UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Control")
            .map_err(DisplayError::InitControlWindow)?;

        debug!("Initializing GPU...");
        let gpu = GpuContext::try_new(&instance, view.surface())?;

        debug!("Configuring windows...");
        let view_window = view.configure(&gpu.adapter, &gpu.device);
        let control_window = control.configure(&gpu.adapter, &gpu.device);

        let gui = GuiContext::new(
            &control_window.window,
            &gpu.device,
            control_window.config.format,
        );

        Ok(Self {
            gpu,
            view: ViewWindow::new(view_window),
            control: ControlWindow::new(control_window, gui),
            params_consumer,
            current_params: VisualParams::default(),
            _feedback_sender: feedback_sender,
        })
    }

    pub fn view_window_id(&self) -> WindowId {
        self.view.window.window.id()
    }

    pub fn control_window_id(&self) -> WindowId {
        self.control.window_id()
    }

    /// Drains the params consumer, keeping only the latest value.
    pub fn drain_params(&mut self) {
        while let Ok(params) = self.params_consumer.pop() {
            self.current_params = params;
        }
    }

    pub fn handle_event(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> Result<bool, DisplayError> {
        let is_view = window_id == self.view_window_id();
        let is_control = window_id == self.control_window_id();

        if is_control && self.control.handle_event(event) {
            return Ok(true);
        }

        match event {
            WindowEvent::Resized(new_size) => {
                if is_view {
                    self.view.resize(&self.gpu.device, *new_size);
                } else if is_control {
                    self.control.resize(&self.gpu.device, *new_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if is_view {
                    self.view.render(&self.gpu, &self.current_params)?;
                } else if is_control {
                    self.control.render(&self.gpu, |ctx| {
                        CentralPanel::default().show(ctx, |ui| {
                            ui.heading("Hello, World!");
                        });
                    })?;
                }
            }

            _ => {}
        }

        Ok(false)
    }
}
