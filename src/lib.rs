use tracing::error;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

mod context;
mod session;

use session::{Session, SessionAction};

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
                return;
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
            match session.update(window_id, event) {
                Ok(Some(action)) => match action {
                    SessionAction::Exit => event_loop.exit(),
                },
                Err(e) => error!("Runtime error: {e}"),
                _ => {}
            }
        }
    }
}
