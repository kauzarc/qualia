//! Audio input for the DSP pipeline.

use std::iter;

use rtrb::Consumer;

use crate::audio::{AudioSamples, HOP_SIZE};

/// Receives and holds audio samples from the audio thread.
pub struct AudioInput {
    consumer: Consumer<AudioSamples>,
    samples: AudioSamples,
}

impl AudioInput {
    pub fn new(consumer: Consumer<AudioSamples>) -> Self {
        Self {
            consumer,
            samples: [0.0; HOP_SIZE],
        }
    }

    /// Drains all pending samples, keeping only the latest.
    pub fn drain_to_latest(&mut self) {
        if let Some(latest) = iter::from_fn(|| self.consumer.pop().ok()).last() {
            self.samples = latest;
        }
    }

    pub fn samples(&self) -> &AudioSamples {
        &self.samples
    }
}
