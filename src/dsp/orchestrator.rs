//! DSP pipeline orchestration.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use rtrb::{Consumer, Producer};
use tracing::{info, warn};

use super::input::AudioInput;
use super::processor::DspProcessor;
use super::state::AudioState;
use super::{HOP_SIZE, SAMPLE_RATE};

/// Sleep duration when no new frame is available.
/// Half a hop at 48kHz (~5.3ms) balances responsiveness with CPU usage.
const IDLE_SLEEP: Duration = Duration::from_micros(1_000_000 * HOP_SIZE as u64 / SAMPLE_RATE / 2);

/// Orchestrates the DSP pipeline: drains input, delegates processing,
/// and pushes output.
pub struct DspOrchestrator {
    input: AudioInput,
    processor: DspProcessor,
    output: Producer<AudioState>,
}

impl DspOrchestrator {
    pub fn new(samples_consumer: Consumer<f64>, output: Producer<AudioState>) -> Self {
        Self {
            input: AudioInput::new(samples_consumer),
            processor: DspProcessor::new(),
            output,
        }
    }

    /// Runs the core loop until the stop flag is set.
    pub fn run(mut self, stop_flag: &AtomicBool) {
        info!("DSP thread started");

        while !stop_flag.load(Ordering::Relaxed) {
            if let Some(samples) = self.input.drain_to_next_hop() {
                let output = self.processor.process(samples);

                if self.output.push(output).is_err() {
                    warn!("AudioState buffer full, dropping frame");
                }
            } else {
                thread::sleep(IDLE_SLEEP);
            }
        }

        info!("DSP thread stopped");
    }
}
