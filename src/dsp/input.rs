//! Audio input for the DSP pipeline.

use std::cmp;

use rtrb::Consumer;

use super::AudioSamples;
use super::accumulator::HopAccumulator;

/// Receives raw audio samples and accumulates them into fixed-size hops.
pub struct AudioInput {
    consumer: Consumer<f64>,
    accumulator: HopAccumulator,
}

impl AudioInput {
    pub fn new(consumer: Consumer<f64>) -> Self {
        Self {
            consumer,
            accumulator: HopAccumulator::new(),
        }
    }

    /// Drains samples until one hop completes or input is exhausted.
    pub fn drain_to_next_hop(&mut self) -> Option<&AudioSamples> {
        let available = self.consumer.slots();
        if available == 0 {
            return None;
        }

        let to_read = cmp::min(available, self.accumulator.remaining());
        let chunk = self
            .consumer
            .read_chunk(to_read)
            .expect("slots() guaranteed availability");
        let hop_complete = <[_; 2]>::from(chunk.as_slices())
            .into_iter()
            .any(|s| self.accumulator.push_slice(s));
        chunk.commit_all();

        hop_complete.then(|| self.accumulator.hop())
    }
}
