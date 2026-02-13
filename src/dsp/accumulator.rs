//! Fixed-size hop accumulator with bulk sample input.

use std::cmp;

use super::{AudioSamples, HOP_SIZE};

/// Accumulates sample slices into fixed-size hops using double-buffering.
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

    /// How many samples are needed to complete the current hop.
    pub fn remaining(&self) -> usize {
        HOP_SIZE - self.pos
    }

    /// Bulk-copies samples into the active buffer, up to `remaining()`.
    /// Returns true when the hop is complete (buffer toggles automatically).
    pub fn push_slice(&mut self, samples: &[f64]) -> bool {
        let to_copy = cmp::min(samples.len(), self.remaining());
        self.buffers[self.active][self.pos..self.pos + to_copy]
            .copy_from_slice(&samples[..to_copy]);
        self.pos += to_copy;

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
