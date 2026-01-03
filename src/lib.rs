use std::sync::Arc;

use egui::Context;
use egui_wgpu::{Renderer, RendererOptions};
use egui_winit::State;
use pollster::FutureExt;
use thiserror::Error;
use tracing::{debug, error};
use wgpu::{
    Adapter, CreateSurfaceError, Device, Instance, InstanceDescriptor, Queue, RequestAdapterError,
    RequestDeviceError, Surface, SurfaceConfiguration, TextureFormat,
};
use winit::{
    application::ApplicationHandler,
    error::OsError,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

/// Main application state orchestrating the GPU and windows.
#[derive(Default)]
pub struct App {
    pub gpu: Option<GpuContext>,

    /// Main visual output.
    pub view_window: Option<WindowContext>,

    /// Controls, graphs, and parameters.
    pub control_window: Option<WindowContext>,

    /// UI Logic attached strictly to the control_window.
    pub gui: Option<GuiContext>,
}

/// Shared GPU resources.
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    fn try_new(instance: &Instance, compatible_surface: &Surface) -> Result<Self, GpuContextError> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(compatible_surface),
                force_fallback_adapter: false,
            })
            .block_on()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Qualia Device"),
                memory_hints: wgpu::MemoryHints::Performance,
                ..Default::default()
            })
            .block_on()?;

        Ok(Self {
            instance: instance.clone(),
            adapter,
            device,
            queue,
        })
    }
}

#[derive(Error, Debug)]
pub enum GpuContextError {
    #[error("wgpu::Adapter request failed: {0}")]
    RequestAdapter(#[from] RequestAdapterError),
    #[error("wgpu::Device request failed: {0}")]
    RequestDevice(#[from] RequestDeviceError),
}

/// A renderable surface associated with a specific OS window.
pub struct WindowContext {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
}

impl WindowContext {
    fn from_raw(
        window: Arc<Window>,
        surface: Surface<'static>,
        adapter: &Adapter,
        device: &Device,
    ) -> Self {
        let size = window.inner_size();
        let caps = surface.get_capabilities(adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Self {
            window,
            surface,
            config,
        }
    }
}

#[derive(Error, Debug)]
pub enum WindowContextError {
    #[error("can't create winit::Window: {0}")]
    CreateWindow(#[from] OsError),
    #[error("can't create wgpu::Surface: {0}")]
    CreateSurface(#[from] CreateSurfaceError),
}

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

impl App {
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

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let instance = Instance::new(&InstanceDescriptor::default());

        debug!("Initializing view window...");
        let (view_window, view_surface) =
            match Self::create_window_and_surface(event_loop, &instance, "Qualia Vision") {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to init view window: {e}");
                    return;
                }
            };

        debug!("Initializing control window...");
        let (control_window, control_surface) =
            match Self::create_window_and_surface(event_loop, &instance, "Qualia Control") {
                Ok(v) => v,
                Err(e) => {
                    error!("Failed to init control window: {e}");
                    return;
                }
            };

        debug!("Initializing GPU...");
        let gpu = match GpuContext::try_new(&instance, &view_surface) {
            Ok(g) => g,
            Err(e) => {
                error!("Failed to init GPU: {e}");
                return;
            }
        };

        let view_context =
            WindowContext::from_raw(view_window, view_surface, &gpu.adapter, &gpu.device);
        let control_context =
            WindowContext::from_raw(control_window, control_surface, &gpu.adapter, &gpu.device);

        let gui_format = control_context.config.format;
        let gui = GuiContext::new(&control_context.window, &gpu.device, gui_format);

        self.view_window = Some(view_context);
        self.control_window = Some(control_context);
        self.gpu = Some(gpu);
        self.gui = Some(gui);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
        todo!()
    }
}
