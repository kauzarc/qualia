//! Audio state input for the inference pipeline.

use std::iter;

use rtrb::Consumer;

use crate::dsp::AudioState;

/// Receives audio state frames and provides the latest available.
pub struct AudioStateInput {
    consumer: Consumer<AudioState>,
}

impl AudioStateInput {
    pub fn new(consumer: Consumer<AudioState>) -> Self {
        Self { consumer }
    }

    /// Drains all available frames, returning the latest.
    pub fn drain_to_latest(&mut self) -> Option<AudioState> {
        iter::from_fn(|| self.consumer.pop().ok()).last()
    }
}
