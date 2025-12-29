use anyhow::Result;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::event_loop::{ControlFlow, EventLoop};

mod app;

use app::App;

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()?;

    info!(version = env!("CARGO_PKG_VERSION"), "Starting");

    debug!("Initializes the event loop");
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    info!("Launch the application");
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
