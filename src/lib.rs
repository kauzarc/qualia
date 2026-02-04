//! Qualia - Real-time visual generation engine driven by online learning.
//!
//! This crate implements an asynchronous multi-threaded architecture separating
//! temporal domains to guarantee low latency on the audio/visual path while
//! allowing background training.

use tracing::error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowId,
};

mod audio;
mod channel;
mod display;
mod dsp;
mod inference;
mod session;
mod trainer;

use session::{Session, SessionAction, SessionEvent};

/// Custom events for cross-thread communication with the event loop.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Inference produced new visual params.
    VisualParamsProduced,
    /// Control panel requests feedback be sent to trainer.
    FeedbackRequested,
}

/// Main application entry point implementing the winit event loop handler.
pub struct App {
    proxy: EventLoopProxy<AppEvent>,
    session: Option<Session>,
}

impl App {
    #[must_use]
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Self {
        Self {
            proxy,
            session: None,
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.is_some() {
            return;
        }

        match Session::try_new(event_loop, self.proxy.clone()) {
            Ok(session) => self.session = Some(session),
            Err(err) => {
                error!("Fatal error initializing session: {err}");
                event_loop.exit();
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        self.handle_session_event(event_loop, SessionEvent::App(event));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        self.handle_session_event(
            event_loop,
            SessionEvent::Window {
                id: window_id,
                event: &event,
            },
        );
    }
}

impl App {
    fn handle_session_event(&mut self, event_loop: &ActiveEventLoop, event: SessionEvent) {
        let Some(session) = &mut self.session else {
            return;
        };

        match session.handle_event(event) {
            Ok(Some(SessionAction::Exit)) => {
                self.session = None;
                event_loop.exit();
            }
            Ok(None) => {}
            Err(e) => error!("Runtime error: {e}"),
        }
    }
}
