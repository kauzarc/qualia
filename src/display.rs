//! Display and rendering module.
//!
//! Manages two windows: the main visual output and the control panel.

mod context;
mod render;

use std::sync::mpsc::Sender;

use rtrb::Consumer;
use thiserror::Error;
use tracing::debug;
use wgpu::{Instance, InstanceDescriptor};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use context::{
    ControlWindow, GpuContext, GpuContextError, GuiContext, UnconfiguredWindow, WindowContext,
    WindowContextError,
};
use render::{ControlRenderer, RenderError, ViewRenderer};

use crate::inference::VisualParams;
use crate::trainer::Feedback;

/// Display subsystem managing windows and GPU rendering.
pub struct Display {
    gpu: GpuContext,
    view: WindowContext,
    control: ControlWindow,
    view_renderer: ViewRenderer,
    control_renderer: ControlRenderer,
    _params_consumer: Consumer<VisualParams>,
    _feedback_sender: Sender<Feedback>,
}

/// Errors that can occur during display operations.
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Failed to init view window: {0}")]
    InitViewWindow(WindowContextError),

    #[error("Failed to init control window: {0}")]
    InitControlWindow(WindowContextError),

    #[error("Failed to init GPU: {0}")]
    InitGpu(#[from] GpuContextError),

    #[error("Error while rendering: {0}")]
    Render(#[from] RenderError),
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
        let view = view.configure(&gpu.adapter, &gpu.device);
        let control_window = control.configure(&gpu.adapter, &gpu.device);
        let gui = GuiContext::new(
            &control_window.window,
            &gpu.device,
            control_window.config.format,
        );

        let control = ControlWindow {
            window: control_window,
            gui,
        };

        Ok(Self {
            gpu,
            view,
            control,
            view_renderer: ViewRenderer,
            control_renderer: ControlRenderer,
            _params_consumer: params_consumer,
            _feedback_sender: feedback_sender,
        })
    }

    pub fn view_window_id(&self) -> WindowId {
        self.view.window.id()
    }

    pub fn control_window_id(&self) -> WindowId {
        self.control.window.window.id()
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
                    self.control.window.resize(&self.gpu.device, *new_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if is_view {
                    self.view_renderer.render(&self.gpu, &self.view)?;
                } else if is_control {
                    self.control_renderer.render(&self.gpu, &mut self.control)?;
                }
            }

            _ => {}
        }

        Ok(false)
    }
}
