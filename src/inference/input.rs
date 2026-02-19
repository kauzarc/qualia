//! Audio state input for the inference pipeline.

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
        let slots = self.consumer.slots();
        if slots == 0 {
            return None;
        }

        let chunk = self
            .consumer
            .read_chunk(slots)
            .expect("slots() guaranteed availability");
        let (first, second) = chunk.as_slices();
        let latest = second.last().or_else(|| first.last()).copied();
        chunk.commit_all();

        latest
    }
}
