use thiserror::Error;
use tracing::debug;
use wgpu::{Instance, InstanceDescriptor};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::{
    audio::{AudioDriver, AudioDriverError},
    context::{
        GpuContext, GpuContextError, GuiContext, UnconfiguredWindow, WindowContext,
        WindowContextError,
    },
    render::{ControlRenderer, RenderError, ViewRenderer},
};

/// Main application state orchestrating the GPU and windows.
pub struct Session {
    audio_driver: AudioDriver,

    gpu: GpuContext,

    /// Main visual output.
    view: WindowContext,

    /// Controls, graphs, and parameters.
    control: WindowContext,

    /// UI Logic attached strictly to the control_window.
    gui: GuiContext,

    view_renderer: ViewRenderer,
    control_renderer: ControlRenderer,
}

#[derive(Debug)]
pub enum SessionAction {
    Exit,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to init audio driver: {0}")]
    InitAudioDriver(#[from] AudioDriverError),

    #[error("Failed to init view window: {0}")]
    InitViewWindow(WindowContextError),

    #[error("Failed to init control window: {0}")]
    InitControlWindow(WindowContextError),

    #[error("Failed to init GPU: {0}")]
    InitGpu(#[from] GpuContextError),

    #[error("Error while rendering view: {0}")]
    RenderView(RenderError),

    #[error("Error while rendering control {0}")]
    RenderControl(RenderError),
}

impl Session {
    pub fn try_new(event_loop: &ActiveEventLoop) -> Result<Self, SessionError> {
        debug!("Starting audio driver...");
        let audio_driver = AudioDriver::try_new()?;

        let instance = Instance::new(&InstanceDescriptor::default());

        debug!("Creating windows...");
        let view = UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Vision")
            .map_err(SessionError::InitViewWindow)?;
        let control = UnconfiguredWindow::try_new(event_loop, &instance, "Qualia Control")
            .map_err(SessionError::InitControlWindow)?;

        debug!("Initializing GPU...");
        let gpu = GpuContext::try_new(&instance, view.surface())?;

        debug!("Configuring windows...");
        let view = view.configure(&gpu.adapter, &gpu.device);
        let control = control.configure(&gpu.adapter, &gpu.device);

        let gui = GuiContext::new(&control.window, &gpu.device, control.config.format);

        Ok(Self {
            audio_driver,
            gpu,
            view,
            control,
            gui,
            view_renderer: ViewRenderer,
            control_renderer: ControlRenderer,
        })
    }

    pub fn update(
        &mut self,
        window_id: WindowId,
        event: WindowEvent,
    ) -> Result<Option<SessionAction>, SessionError> {
        if window_id == self.control.window.id() {
            let response = self.gui.state.on_window_event(&self.control.window, &event);

            if response.consumed {
                return Ok(None);
            }
        }

        match event {
            WindowEvent::CloseRequested => Ok(Some(SessionAction::Exit)),

            WindowEvent::Resized(new_size) => {
                if window_id == self.view.window.id() {
                    self.view.resize(&self.gpu.device, new_size);
                } else if window_id == self.control.window.id() {
                    self.control.resize(&self.gpu.device, new_size);
                }

                Ok(None)
            }

            WindowEvent::RedrawRequested => {
                if window_id == self.view.window.id() {
                    self.view_renderer
                        .render(&self.gpu, &self.view)
                        .map_err(SessionError::RenderView)?;
                } else if window_id == self.control.window.id() {
                    self.control_renderer
                        .render(&self.gpu, &self.control, &mut self.gui)
                        .map_err(SessionError::RenderControl)?;
                }

                Ok(None)
            }

            _ => Ok(None),
        }
    }
}
