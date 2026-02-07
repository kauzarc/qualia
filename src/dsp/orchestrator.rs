//! DSP pipeline orchestration.

use std::sync::atomic::{AtomicBool, Ordering};

use rtrb::{Consumer, Producer};
use tracing::{info, warn};

use crate::audio::AudioSamples;

use super::input::AudioInput;
use super::processor::DspProcessor;
use super::state::AudioState;

/// Orchestrates the DSP pipeline: drains input, delegates processing,
/// and pushes output.
pub struct DspOrchestrator {
    input: AudioInput,
    processor: DspProcessor,
    output: Producer<AudioState>,
}

impl DspOrchestrator {
    pub fn new(samples_consumer: Consumer<AudioSamples>, output: Producer<AudioState>) -> Self {
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
            self.input.drain_to_latest();

            let output = self.processor.process(self.input.samples());

            if self.output.push(output).is_err() {
                warn!("AudioState buffer full, dropping frame");
            }
        }

        info!("DSP thread stopped");
    }
}
