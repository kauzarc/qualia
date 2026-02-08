//! Audio input for the DSP pipeline.

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
        while let Ok(sample) = self.consumer.pop() {
            if self.accumulator.push(sample) {
                return Some(self.accumulator.hop());
            }
        }

        None
    }
}
