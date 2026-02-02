use anyhow::Result;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::event_loop::{ControlFlow, EventLoop};

use qualia::{App, AppEvent};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()?;

    info!(version = env!("CARGO_PKG_VERSION"), "Starting");

    debug!("Initializes the event loop");
    let event_loop = EventLoop::<AppEvent>::with_user_event().build()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let proxy = event_loop.create_proxy();

    info!("Launch the application");
    let mut app = App::new(proxy);
    event_loop.run_app(&mut app)?;

    Ok(())
}
