//! Visual params buffering and interpolation.

use std::time::{Duration, Instant};

use rtrb::Consumer;

use super::ring_pair::RingPair;
use crate::inference::{ControlVoltage, VisualParams};

/// Assumed display latency.
const DISPLAY_DELAY: Duration = Duration::from_millis(16);

/// Buffer for visual params with time-based interpolation.
pub struct ParamsBuffer {
    consumer: Consumer<VisualParams>,
    buffer: RingPair<VisualParams>,
}

impl ParamsBuffer {
    pub fn new(consumer: Consumer<VisualParams>) -> Self {
        Self {
            consumer,
            buffer: RingPair::default(),
        }
    }

    /// Consumes all available values.
    pub fn update(&mut self) {
        while let Ok(params) = self.consumer.pop() {
            self.buffer.push(params);
        }
    }

    /// Computes interpolated control voltages based on timestamps and display delay.
    pub fn interpolated_actions(&self) -> Box<[ControlVoltage]> {
        let older = self.buffer.older();
        let newer = self.buffer.newer();
        let n = newer.num_actions;

        if newer.is_transient {
            return newer.actions[..n].into();
        }

        let render_time = Instant::now().saturating_duration_since(newer.timestamp);

        if render_time >= DISPLAY_DELAY {
            return newer.actions[..n].into();
        }

        let duration = newer.timestamp.saturating_duration_since(older.timestamp);

        if duration.is_zero() {
            return newer.actions[..n].into();
        }

        let elapsed = DISPLAY_DELAY - render_time;
        let t = 1.0 - elapsed.as_secs_f64() / duration.as_secs_f64();

        older.actions[..n]
            .iter()
            .zip(&newer.actions[..n])
            .map(|(a, b)| ControlVoltage::clamped(a.get() + (b.get() - a.get()) * t))
            .collect()
    }
}
