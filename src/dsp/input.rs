//! Audio input for the DSP pipeline.

use rtrb::Consumer;

use super::{AudioSamples, HOP_SIZE};

/// Receives raw audio samples and accumulates them into fixed-size frames.
pub struct AudioInput {
    consumer: Consumer<f64>,
    accumulator: [f64; HOP_SIZE],
    pos: usize,
    samples: AudioSamples,
}

impl AudioInput {
    pub fn new(consumer: Consumer<f64>) -> Self {
        Self {
            consumer,
            accumulator: [0.0; HOP_SIZE],
            pos: 0,
            samples: [0.0; HOP_SIZE],
        }
    }

    /// Drains all available samples, accumulating into frames
    /// and keeping only the latest complete frame.
    pub fn drain_to_latest(&mut self) {
        while let Ok(sample) = self.consumer.pop() {
            self.accumulator[self.pos] = sample;
            self.pos += 1;

            if self.pos == HOP_SIZE {
                self.pos = 0;
                self.samples = self.accumulator;
            }
        }
    }

    pub fn samples(&self) -> &AudioSamples {
        &self.samples
    }
}
