use crate::audio::{AudioSamples, HOP_SIZE};

use super::MEL_BANDS;
use super::state::AudioState;

/// Threshold for transient detection based on energy delta.
const TRANSIENT_THRESHOLD: f32 = 0.1;

/// Transforms audio samples into audio state.
pub trait Processor {
    fn process(&mut self, samples: AudioSamples) -> AudioState;
}

/// Handles DSP computations: FFT, Mel spectrogram, and feature extraction.
pub struct DspProcessor {
    timestamp: u64,
    prev_energy: f32,
}

impl DspProcessor {
    pub fn new() -> Self {
        Self {
            timestamp: 0,
            prev_energy: 0.0,
        }
    }
}

impl Processor for DspProcessor {
    fn process(&mut self, samples: AudioSamples) -> AudioState {
        let energy = self.compute_energy(&samples);
        let mel_bands = self.compute_mel_bands(&samples);
        let spectral_flux = self.compute_spectral_flux(&samples);
        let zero_crossing_rate = self.compute_zero_crossing_rate(&samples);
        let is_transient = self.detect_transient(energy);

        self.prev_energy = energy;
        let timestamp = self.timestamp;
        self.timestamp = self.timestamp.wrapping_add(1);

        AudioState {
            mel_bands,
            energy,
            spectral_flux,
            zero_crossing_rate,
            is_transient,
            timestamp,
        }
    }
}

impl DspProcessor {
    fn compute_energy(&self, samples: &AudioSamples) -> f32 {
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / HOP_SIZE as f32).sqrt()
    }

    fn compute_mel_bands(&self, _samples: &AudioSamples) -> [f32; MEL_BANDS] {
        // TODO: FFT -> Mel filterbank
        [0.0; MEL_BANDS]
    }

    fn compute_spectral_flux(&self, _samples: &AudioSamples) -> f32 {
        // TODO: compute spectral flux from FFT magnitude difference
        0.0
    }

    fn compute_zero_crossing_rate(&self, _samples: &AudioSamples) -> f32 {
        // TODO: count zero crossings / sample count
        0.0
    }

    fn detect_transient(&self, energy: f32) -> bool {
        energy - self.prev_energy > TRANSIENT_THRESHOLD
    }
}
