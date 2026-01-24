use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, error};
use wgpu::{Instance, InstanceDescriptor, Surface};
use winit::{
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::context::{GpuContext, GpuContextError, GuiContext, WindowContext, WindowContextError};

/// Main application state orchestrating the GPU and windows.
pub struct Session {
    pub gpu: GpuContext,

    /// Main visual output.
    pub view: WindowContext,

    /// Controls, graphs, and parameters.
    pub control: WindowContext,

    /// UI Logic attached strictly to the control_window.
    pub gui: GuiContext,
}

#[derive(Debug)]
pub enum SessionAction {
    Exit,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to init view window: {0}")]
    InitViewWindow(WindowContextError),

    #[error("Failed to init control window: {0}")]
    InitControlWindow(WindowContextError),

    #[error("Failed to init GPU: {0}")]
    InitGpu(#[from] GpuContextError),
}

impl Session {
    pub fn try_new(event_loop: &ActiveEventLoop) -> Result<Self, SessionError> {
        let instance = Instance::new(&InstanceDescriptor::default());

        debug!("Initializing view window...");
        let (view_window, view_surface) =
            Self::create_window_and_surface(event_loop, &instance, "Qualia Vision")
                .map_err(SessionError::InitViewWindow)?;

        debug!("Initializing control window...");
        let (control_window, control_surface) =
            Self::create_window_and_surface(event_loop, &instance, "Qualia Control")
                .map_err(SessionError::InitControlWindow)?;

        debug!("Initializing GPU...");
        let gpu = GpuContext::try_new(&instance, &view_surface)?;

        let view_context =
            WindowContext::from_raw(view_window, view_surface, &gpu.adapter, &gpu.device);
        let control_context =
            WindowContext::from_raw(control_window, control_surface, &gpu.adapter, &gpu.device);

        let gui_format = control_context.config.format;
        let gui = GuiContext::new(&control_context.window, &gpu.device, gui_format);

        Ok(Self {
            gpu,
            view: view_context,
            control: control_context,
            gui,
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
                    self.render_view();
                } else if window_id == self.control.window.id() {
                    self.render_control();
                }

                Ok(None)
            }

            _ => Ok(None),
        }
    }

    fn render_view(&mut self) -> () {
        // View render logic...
        self.view.window.request_redraw();
    }

    fn render_control(&mut self) -> () {
        // Gui render logic..
        self.control.window.request_redraw();
    }

    fn create_window_and_surface(
        event_loop: &ActiveEventLoop,
        instance: &Instance,
        title: &str,
    ) -> Result<(Arc<Window>, Surface<'static>), WindowContextError> {
        let attr = Window::default_attributes().with_title(title);
        let window = Arc::new(event_loop.create_window(attr)?);
        let surface = instance.create_surface(window.clone())?;
        Ok((window, surface))
    }
}
