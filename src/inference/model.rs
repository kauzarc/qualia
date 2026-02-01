use crate::dsp::AudioState;
use crate::trainer::Reward;

use super::params::VisualParams;

/// Neural network model that transforms audio state into visual parameters.
pub trait InferenceModel {
    /// Number of output actions (must match shader inputs).
    fn _output_size(&self) -> usize;

    /// Forward pass: audio state to visual parameters.
    fn infer(&mut self, state: &AudioState) -> VisualParams;

    /// Receive reward signal for learning.
    fn _reward(&mut self, reward: Reward);
}
