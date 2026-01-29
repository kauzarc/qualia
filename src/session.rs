use thiserror::Error;
use tracing::debug;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::{
    audio::{AudioDriver, AudioDriverError},
    display::{Display, DisplayError},
};

pub struct Session {
    audio_driver: AudioDriver,
    display: Display,
}

#[derive(Debug)]
pub enum SessionAction {
    Exit,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to init audio driver: {0}")]
    InitAudioDriver(#[from] AudioDriverError),

    #[error("Failed to init display: {0}")]
    InitDisplay(#[from] DisplayError),
}

impl Session {
    pub fn try_new(event_loop: &ActiveEventLoop) -> Result<Self, SessionError> {
        debug!("Starting audio driver...");
        let audio_driver = AudioDriver::try_new()?;

        debug!("Initializing display...");
        let display = Display::try_new(event_loop)?;

        Ok(Self {
            audio_driver,
            display,
        })
    }

    pub fn update(
        &mut self,
        window_id: WindowId,
        event: WindowEvent,
    ) -> Result<Option<SessionAction>, SessionError> {
        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(Some(SessionAction::Exit));
        }

        self.display
            .handle_event(window_id, &event)
            .map_err(SessionError::from)?;

        Ok(None)
    }
}
