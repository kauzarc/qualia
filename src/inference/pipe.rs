//! Inference pipeline IO.

use std::iter;

use rtrb::{Consumer, Producer};

use crate::dsp::AudioState;

use super::model::InferenceModel;
use super::params::VisualParams;

/// Result of a single pipeline tick.
pub enum TickResult {
    Produced,
    NoInput,
    BufferFull,
}

/// Connects audio state input to visual params output via model inference.
pub struct InferencePipe<M> {
    model: M,
    input: Consumer<AudioState>,
    output: Producer<VisualParams>,
}

impl<M: InferenceModel> InferencePipe<M> {
    pub fn new(model: M, input: Consumer<AudioState>, output: Producer<VisualParams>) -> Self {
        Self {
            model,
            input,
            output,
        }
    }

    /// Processes the latest available input.
    pub fn tick(&mut self) -> TickResult {
        let Some(input) = self.drain_to_latest() else {
            return TickResult::NoInput;
        };

        let output = self.model.infer(&input);

        match self.output.push(output) {
            Ok(()) => TickResult::Produced,
            Err(_) => TickResult::BufferFull,
        }
    }

    fn drain_to_latest(&mut self) -> Option<AudioState> {
        iter::from_fn(|| self.input.pop().ok()).last()
    }
}
