use crate::channel::Processor;
use crate::dsp::AudioState;

use super::model::InferenceModel;
use super::params::VisualParams;

/// Processor that wraps an `InferenceModel` for use with `Pipe`.
pub struct InferenceProcessor<M> {
    model: M,
}

impl<M> InferenceProcessor<M> {
    pub fn new(model: M) -> Self {
        Self { model }
    }
}

impl<M: InferenceModel> Processor for InferenceProcessor<M> {
    type Input = AudioState;
    type Output = VisualParams;

    fn process(&mut self, input: AudioState) -> VisualParams {
        self.model.infer(&input)
    }
}
