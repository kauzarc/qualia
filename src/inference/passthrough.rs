use crate::dsp::AudioState;
use crate::trainer::Reward;

use super::model::InferenceModel;
use super::params::{ControlVoltage, MAX_ACTIONS, VisualParams};

/// Passthrough model that maps audio features directly to visual params.
///
/// MVP placeholder for the real neural network.
pub struct PassthroughModel {
    output_size: usize,
}

impl PassthroughModel {
    pub fn new(output_size: usize) -> Self {
        Self { output_size }
    }
}

impl InferenceModel for PassthroughModel {
    fn _output_size(&self) -> usize {
        self.output_size
    }

    fn infer(&mut self, state: &AudioState) -> VisualParams {
        let mut actions = [ControlVoltage::default(); MAX_ACTIONS];

        // Map energy to first action (brightness/intensity)
        actions[0] = ControlVoltage::clamped(state.energy);

        // Map some mel bands to RGB-ish controls
        actions[1] = ControlVoltage::clamped(state.mel_bands[0]); // low freq
        actions[2] = ControlVoltage::clamped(state.mel_bands[32]); // mid freq
        actions[3] = ControlVoltage::clamped(state.mel_bands[63]); // high freq

        VisualParams {
            _actions: actions,
            _num_actions: self.output_size,
            _is_transient: state.is_transient,
            _timestamp: state.timestamp,
        }
    }

    fn _reward(&mut self, _reward: Reward) {
        // Passthrough model ignores rewards
    }
}
