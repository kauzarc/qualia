//! DSP pipeline IO.

use std::iter;

use rtrb::{Consumer, Producer};

use crate::audio::AudioSamples;

use super::processor::DspProcessor;
use super::state::AudioState;

/// Result of a single pipeline tick.
pub enum TickResult {
    Produced,
    NoInput,
    BufferFull,
}

/// Connects audio input to state output via DSP processing.
pub struct DspPipe {
    processor: DspProcessor,
    input: Consumer<AudioSamples>,
    output: Producer<AudioState>,
}

impl DspPipe {
    pub fn new(
        processor: DspProcessor,
        input: Consumer<AudioSamples>,
        output: Producer<AudioState>,
    ) -> Self {
        Self {
            processor,
            input,
            output,
        }
    }

    /// Processes the latest available input.
    pub fn tick(&mut self) -> TickResult {
        let Some(input) = self.drain_to_latest() else {
            return TickResult::NoInput;
        };

        let output = self.processor.process(&input);

        match self.output.push(output) {
            Ok(()) => TickResult::Produced,
            Err(_) => TickResult::BufferFull,
        }
    }

    fn drain_to_latest(&mut self) -> Option<AudioSamples> {
        iter::from_fn(|| self.input.pop().ok()).last()
    }
}
