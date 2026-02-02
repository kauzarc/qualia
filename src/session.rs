//! Session orchestration module.
//!
//! Creates and connects all threads and communication channels.

use thiserror::Error;
use winit::{
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{
    AppEvent,
    audio::{AudioDriver, AudioDriverError},
    channel::Channels,
    display::{Display, DisplayError},
    dsp::{DspEngine, DspEngineError},
    inference::{Inference, InferenceError},
    trainer::{Trainer, TrainerError},
};

/// Active session holding all threads and the display.
pub struct Session {
    _audio_driver: AudioDriver,
    _dsp_engine: DspEngine,
    _inference: Inference,
    _trainer: Trainer,
    display: Display,
}

/// Events that the session can handle.
pub enum SessionEvent<'a> {
    /// A window event from winit.
    Window {
        id: WindowId,
        event: &'a WindowEvent,
    },
    /// A custom application event.
    App(AppEvent),
}

/// Actions that can be requested by the session.
#[derive(Debug)]
pub enum SessionAction {
    Exit,
}

/// Errors that can occur when initializing a session.
#[derive(Debug, Error)]
pub enum SessionInitError {
    #[error("Failed to init audio driver: {0}")]
    AudioDriver(#[from] AudioDriverError),

    #[error("Failed to init DSP engine: {0}")]
    DspEngine(#[from] DspEngineError),

    #[error("Failed to init inference: {0}")]
    Inference(#[from] InferenceError),

    #[error("Failed to init trainer: {0}")]
    Trainer(#[from] TrainerError),

    #[error("Failed to init display: {0}")]
    Display(#[from] DisplayError),
}

impl Session {
    /// Creates a new session, initializing all threads and channels.
    pub fn try_new(
        event_loop: &ActiveEventLoop,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<Self, SessionInitError> {
        let channels = Channels::new();

        let audio_driver = AudioDriver::try_new(channels.samples_producer)?;
        let dsp_engine = DspEngine::try_new(channels.samples_consumer, channels.state_producer)?;
        let inference =
            Inference::try_new(channels.state_consumer, channels.params_producer, proxy)?;
        let trainer = Trainer::try_new(channels.feedback_receiver)?;
        let display = Display::try_new(
            event_loop,
            channels.params_consumer,
            channels.feedback_sender,
        )?;

        Ok(Self {
            _audio_driver: audio_driver,
            _dsp_engine: dsp_engine,
            _inference: inference,
            _trainer: trainer,
            display,
        })
    }

    /// Handles an event, returning an action if needed.
    pub fn handle_event(
        &mut self,
        event: SessionEvent,
    ) -> Result<Option<SessionAction>, DisplayError> {
        match event {
            SessionEvent::Window { id, event } => {
                if matches!(event, WindowEvent::CloseRequested) {
                    return Ok(Some(SessionAction::Exit));
                }

                self.display.handle_event(id, event)?;
            }

            SessionEvent::App(AppEvent::VisualParamsProduced) => {
                self.display.drain_params();
            }
        }
        Ok(None)
    }
}
