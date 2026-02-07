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
    pub samples_producer: Producer<f64>,
    pub samples_consumer: Consumer<f64>,
    pub state_producer: Producer<AudioState>,
    pub state_consumer: Consumer<AudioState>,
    pub params_producer: Producer<VisualParams>,
    pub params_consumer: Consumer<VisualParams>,
    pub feedback_sender: Sender<Feedback>,
    pub feedback_receiver: Receiver<Feedback>,
}

impl Channels {
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
