//! Display and rendering module.
//!
//! Manages two windows: the main visual output and the control panel.

mod control;
mod gpu;
mod view;
mod window;

use std::sync::mpsc::Sender;
use std::time::{SystemTime, UNIX_EPOCH};

use rtrb::Consumer;
use thiserror::Error;
use tracing::debug;
use wgpu::{Instance, InstanceDescriptor};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::WindowId;

use control::ControlWindow;
pub use gpu::GpuContext;
use gpu::{GpuContextError, RenderError};
use view::ViewWindow;
use window::{UnconfiguredWindow, WindowError};

use crate::AppEvent;
use crate::inference::VisualParams;
use crate::trainer::{Feedback, Reward};

/// Display subsystem managing windows and GPU rendering.
pub struct Display {
    gpu: GpuContext,
    view: ViewWindow,
    control: ControlWindow,
    params_consumer: Consumer<VisualParams>,
    current_params: VisualParams,
    feedback_sender: Sender<Feedback>,
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

    #[error("Trainer disconnected")]
    TrainerDisconnected,
}

impl Display {
    pub fn try_new(
        event_loop: &ActiveEventLoop,
        params_consumer: Consumer<VisualParams>,
        feedback_sender: Sender<Feedback>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<Self, DisplayError> {
        let instance = Instance::new(&InstanceDescriptor::default());

        debug!("Creating windows...");
        let view_unconfigured = UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Vision")
            .map_err(DisplayError::InitViewWindow)?;
        let control_unconfigured =
            UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Control")
                .map_err(DisplayError::InitControlWindow)?;

        debug!("Initializing GPU...");
        let gpu = GpuContext::try_new(&instance, view_unconfigured.surface())?;

        debug!("Configuring windows...");
        let view = ViewWindow::new(view_unconfigured, &gpu.adapter, &gpu.device);
        let control = ControlWindow::new(control_unconfigured, &gpu.adapter, &gpu.device, proxy);

        Ok(Self {
            gpu,
            view,
            control,
            params_consumer,
            current_params: VisualParams::default(),
            feedback_sender,
        })
    }

    pub fn view_window_id(&self) -> WindowId {
        self.view.window.window.id()
    }

    pub fn control_window_id(&self) -> WindowId {
        self.control.window_id()
    }

    /// Updates visual params by consuming all available values, keeping only the latest.
    pub fn update_visual_params(&mut self) {
        while let Ok(params) = self.params_consumer.pop() {
            self.current_params = params;
        }
    }

    /// Sends feedback to the trainer.
    pub fn send_feedback(&self) -> Result<(), DisplayError> {
        let feedback = Feedback {
            reward: Reward::new(0.0).expect("0.0 is valid"),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
                .unwrap_or(0),
        };
        self.feedback_sender
            .send(feedback)
            .map_err(|_| DisplayError::TrainerDisconnected)
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

        match *event {
            WindowEvent::Resized(new_size) => {
                if is_view {
                    self.view.resize(&self.gpu.device, new_size);
                } else if is_control {
                    self.control.resize(&self.gpu.device, new_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if is_view {
                    self.view.render(&self.gpu, &self.current_params)?;
                } else if is_control {
                    self.control.render(&self.gpu)?;
                }
            }

            _ => {}
        }

        Ok(false)
    }
}
