use std::sync::mpsc::{self, Receiver, Sender};

use rtrb::{Consumer, Producer};
use tracing::debug;

use crate::dsp::{AudioState, HOP_SIZE};
use crate::inference::VisualParams;
use crate::trainer::Feedback;

const DSP_RATE_HZ: usize = 90;
const INFERENCE_RATE_HZ: usize = 60;
const AUDIO_TOLERANCE_MS: usize = 90;
const VISUAL_TOLERANCE_MS: usize = 67;

const fn buffer_capacity(rate_hz: usize, tolerance_ms: usize) -> usize {
    (rate_hz * tolerance_ms) / 1000
}

const SAMPLES_BUFFER_CAPACITY: usize = buffer_capacity(DSP_RATE_HZ, AUDIO_TOLERANCE_MS) * HOP_SIZE;
const STATE_BUFFER_CAPACITY: usize = buffer_capacity(DSP_RATE_HZ, AUDIO_TOLERANCE_MS);
const PARAMS_BUFFER_CAPACITY: usize = buffer_capacity(INFERENCE_RATE_HZ, VISUAL_TOLERANCE_MS);

/// All communication channels between pipeline components.
pub struct Channels {
    /// Sends raw audio samples from Audio → DSP.
    pub samples_producer: Producer<f64>,
    /// Receives raw audio samples in the DSP thread.
    pub samples_consumer: Consumer<f64>,
    /// Sends extracted audio state from DSP → Inference.
    pub state_producer: Producer<AudioState>,
    /// Receives audio state in the Inference thread.
    pub state_consumer: Consumer<AudioState>,
    /// Sends visual params from Inference → Display.
    pub params_producer: Producer<VisualParams>,
    /// Receives visual params in the Display thread.
    pub params_consumer: Consumer<VisualParams>,
    /// Sends user feedback from Display → Trainer.
    pub feedback_sender: Sender<Feedback>,
    /// Receives user feedback in the Trainer thread.
    pub feedback_receiver: Receiver<Feedback>,
}

impl Channels {
    /// Creates all pipeline channels with pre-computed buffer capacities.
    pub fn new() -> Self {
        debug!("Creating communication channels");

        let (samples_producer, samples_consumer) =
            rtrb::RingBuffer::<f64>::new(SAMPLES_BUFFER_CAPACITY);
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
