use rtrb::{Consumer, Producer};
use tracing::warn;

use crate::audio::AudioSamples;

use super::processor::Processor;
use super::state::AudioState;

/// Connects two ring buffers, transforming input to output.
///
/// Uses "drain to latest" strategy: discards stale inputs, processes only the freshest.
pub struct Pipe<P> {
    processor: P,
    input: Consumer<AudioSamples>,
    output: Producer<AudioState>,
}

impl<P: Processor> Pipe<P> {
    pub fn new(processor: P, input: Consumer<AudioSamples>, output: Producer<AudioState>) -> Self {
        Self {
            processor,
            input,
            output,
        }
    }

    /// Processes the latest available input.
    ///
    /// Returns `true` if something was processed, `false` if input was empty.
    pub fn tick(&mut self) -> bool {
        let Some(samples) = self.drain_to_latest() else {
            return false;
        };

        let state = self.processor.process(samples);

        if self.output.push(state).is_err() {
            warn!("Pipe output buffer full, dropping frame");
        }

        true
    }

    fn drain_to_latest(&mut self) -> Option<AudioSamples> {
        let mut latest = None;
        while let Ok(sample) = self.input.pop() {
            latest = Some(sample);
        }
        latest
    }
}
