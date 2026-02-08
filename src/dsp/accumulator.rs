//! Fixed-size hop accumulator for sample-by-sample input.

use super::{AudioSamples, HOP_SIZE};

/// Accumulates individual samples into fixed-size hops using double-buffering.
///
/// One buffer is actively written to while the other holds the latest complete hop.
/// When the active buffer fills, the index is toggled — no copy needed.
pub struct HopAccumulator {
    buffers: [AudioSamples; 2],
    active: usize,
    pos: usize,
}

impl HopAccumulator {
    pub fn new() -> Self {
        Self {
            buffers: [[0.0; HOP_SIZE]; 2],
            active: 0,
            pos: 0,
        }
    }

    /// Pushes a single sample. Returns `true` when a new hop is complete.
    pub fn push(&mut self, sample: f64) -> bool {
        self.buffers[self.active][self.pos] = sample;
        self.pos += 1;

        if self.pos == HOP_SIZE {
            self.pos = 0;
            self.active ^= 1;
            return true;
        }

        false
    }

    pub fn hop(&self) -> &AudioSamples {
        &self.buffers[self.active ^ 1]
    }
}
