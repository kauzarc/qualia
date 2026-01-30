//! Qualia - Real-time visual generation engine driven by online learning.
//!
//! This crate implements an asynchronous multi-threaded architecture separating
//! temporal domains to guarantee low latency on the audio/visual path while
//! allowing background training.

use tracing::error;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

mod audio;
mod display;
mod dsp;
mod inference;
mod session;
mod trainer;

use session::{Session, SessionAction};

/// Main application entry point implementing the winit event loop handler.
#[derive(Default)]
pub struct App {
    session: Option<Session>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.is_some() {
            return;
        }

        match Session::try_new(event_loop) {
            Ok(session) => self.session = Some(session),
            Err(err) => {
                error!("Fatal error initializing session: {err}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(session) = &mut self.session {
            match session.update(window_id, &event) {
                Ok(Some(action)) => match action {
                    SessionAction::Exit => {
                        self.session = None;
                        event_loop.exit();
                    }
                },
                Err(e) => error!("Runtime error: {e}"),
                _ => {}
            }
        }
    }
}
