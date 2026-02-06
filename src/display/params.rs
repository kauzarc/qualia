//! Visual params buffering and interpolation.

use std::time::{SystemTime, UNIX_EPOCH};

use rtrb::Consumer;

use super::ring_pair::RingPair;
use crate::inference::{ControlVoltage, VisualParams};

/// Assumed display latency in milliseconds.
const DISPLAY_DELAY_MS: u64 = 16;

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
    #[expect(
        clippy::cast_precision_loss,
        reason = "interpolation factor t only needs rough precision"
    )]
    pub fn interpolated_actions(&self) -> Box<[ControlVoltage]> {
        let older = self.buffer.older();
        let newer = self.buffer.newer();
        let n = newer.num_actions;

        if newer.is_transient {
            return newer.actions[..n].into();
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX));

        let render_time = now.saturating_sub(DISPLAY_DELAY_MS);
        let duration = newer.timestamp.saturating_sub(older.timestamp);

        if duration == 0 || render_time >= newer.timestamp {
            return newer.actions[..n].into();
        }

        let t = render_time.saturating_sub(older.timestamp) as f64 / duration as f64;

        older.actions[..n]
            .iter()
            .zip(&newer.actions[..n])
            .map(|(a, b)| ControlVoltage::clamped(a.get() + (b.get() - a.get()) * t))
            .collect()
    }
}
