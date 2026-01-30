use std::sync::mpsc::{self, Receiver, Sender};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::debug;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use crate::{
    audio::{AudioDriver, AudioDriverError, AudioSamples},
    display::{Display, DisplayError},
    dsp::{AudioState, DspEngine, DspEngineError},
    inference::{Inference, InferenceError, VisualParams},
    trainer::{Feedback, Trainer, TrainerError},
};

pub struct Session {
    _audio_driver: AudioDriver,
    _dsp_engine: DspEngine,
    _inference: Inference,
    _trainer: Trainer,
    display: Display,
}

struct Channels {
    samples_producer: Producer<AudioSamples>,
    samples_consumer: Consumer<AudioSamples>,
    state_producer: Producer<AudioState>,
    state_consumer: Consumer<AudioState>,
    params_producer: Producer<VisualParams>,
    params_consumer: Consumer<VisualParams>,
    feedback_sender: Sender<Feedback>,
    feedback_receiver: Receiver<Feedback>,
}

impl Channels {
    fn new() -> Self {
        debug!("Creating communication channels");
        let (samples_producer, samples_consumer) = rtrb::RingBuffer::<AudioSamples>::new(8);
        let (state_producer, state_consumer) = rtrb::RingBuffer::<AudioState>::new(8);
        let (params_producer, params_consumer) = rtrb::RingBuffer::<VisualParams>::new(4);
        let (feedback_sender, feedback_receiver) = mpsc::channel();

        Self {
            samples_producer,
            samples_consumer,
            state_producer,
            state_consumer,
            params_producer,
            params_consumer,
            feedback_sender,
            feedback_receiver,
        }
    }
}

#[derive(Debug)]
pub enum SessionAction {
    Exit,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("Failed to init audio driver: {0}")]
    InitAudioDriver(#[from] AudioDriverError),

    #[error("Failed to init DSP engine: {0}")]
    InitDspEngine(#[from] DspEngineError),

    #[error("Failed to init inference: {0}")]
    InitInference(#[from] InferenceError),

    #[error("Failed to init trainer: {0}")]
    InitTrainer(#[from] TrainerError),

    #[error("Failed to init display: {0}")]
    InitDisplay(#[from] DisplayError),
}

impl Session {
    pub fn try_new(event_loop: &ActiveEventLoop) -> Result<Self, SessionError> {
        let channels = Channels::new();

        let audio_driver = AudioDriver::try_new(channels.samples_producer)?;
        let dsp_engine = DspEngine::try_new(channels.samples_consumer, channels.state_producer)?;
        let inference = Inference::try_new(channels.state_consumer, channels.params_producer)?;
        let trainer = Trainer::try_new(channels.feedback_receiver)?;
        let display = Display::try_new(event_loop, channels.params_consumer, channels.feedback_sender)?;

        Ok(Self {
            _audio_driver: audio_driver,
            _dsp_engine: dsp_engine,
            _inference: inference,
            _trainer: trainer,
            display,
        })
    }

    pub fn update(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> Result<Option<SessionAction>, SessionError> {
        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(Some(SessionAction::Exit));
        }

        self.display
            .handle_event(window_id, event)
            .map_err(SessionError::from)?;

        Ok(None)
    }
}
