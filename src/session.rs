use std::sync::mpsc::{self, Receiver, Sender};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::debug;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

const DSP_RATE_HZ: usize = 90;
const INFERENCE_RATE_HZ: usize = 60;
const AUDIO_TOLERANCE_MS: usize = 90;
const VISUAL_TOLERANCE_MS: usize = 67;

const fn buffer_capacity(rate_hz: usize, tolerance_ms: usize) -> usize {
    (rate_hz * tolerance_ms) / 1000
}

const SAMPLES_BUFFER_CAPACITY: usize = buffer_capacity(DSP_RATE_HZ, AUDIO_TOLERANCE_MS);
const STATE_BUFFER_CAPACITY: usize = buffer_capacity(DSP_RATE_HZ, AUDIO_TOLERANCE_MS);
const PARAMS_BUFFER_CAPACITY: usize = buffer_capacity(INFERENCE_RATE_HZ, VISUAL_TOLERANCE_MS);

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
        let (samples_producer, samples_consumer) =
            rtrb::RingBuffer::<AudioSamples>::new(SAMPLES_BUFFER_CAPACITY);
        let (state_producer, state_consumer) =
            rtrb::RingBuffer::<AudioState>::new(STATE_BUFFER_CAPACITY);
        let (params_producer, params_consumer) =
            rtrb::RingBuffer::<VisualParams>::new(PARAMS_BUFFER_CAPACITY);
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
    pub fn try_new(event_loop: &ActiveEventLoop) -> Result<Self, SessionInitError> {
        let channels = Channels::new();

        let audio_driver = AudioDriver::try_new(channels.samples_producer)?;
        let dsp_engine = DspEngine::try_new(channels.samples_consumer, channels.state_producer)?;
        let inference = Inference::try_new(channels.state_consumer, channels.params_producer)?;
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

    pub fn update(
        &mut self,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> Result<Option<SessionAction>, DisplayError> {
        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(Some(SessionAction::Exit));
        }

        self.display.handle_event(window_id, event)?;

        Ok(None)
    }
}
