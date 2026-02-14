mod fft;
mod mel;

use std::time::Instant;

use self::fft::Fft;
use self::mel::MelFilterbank;
use super::state::AudioState;
use super::{AudioSamples, HOP_SIZE, SAMPLE_RATE};
use crate::ring_pair::RingPair;

/// Number of FFT frequency bins (`HOP_SIZE / 2 + 1`).
const SPECTRUM_SIZE: usize = HOP_SIZE / 2 + 1;

/// Threshold for transient detection based on energy delta.
const TRANSIENT_THRESHOLD: f64 = 0.1;

/// Handles DSP computations: FFT, Mel spectrogram, and feature extraction.
pub struct DspProcessor {
    prev_energy: f64,
    fft: Fft,
    mel_filterbank: MelFilterbank,
    spectrums: RingPair<[f64; SPECTRUM_SIZE]>,
}

impl DspProcessor {
    /// Initializes FFT, mel filterbank, and spectrum history for stateful processing.
    pub fn new() -> Self {
        Self {
            prev_energy: 0.0,
            fft: Fft::new(),
            mel_filterbank: MelFilterbank::new(SAMPLE_RATE),
            spectrums: RingPair::new([0.0; SPECTRUM_SIZE]),
        }
    }

    /// Extracts all audio features from one hop of samples into an [`AudioState`].
    pub fn process(&mut self, samples: &AudioSamples) -> AudioState {
        self.spectrums
            .push_with(|buffer| self.fft.power_spectrum(samples, buffer));
        let energy = Self::compute_energy(samples);
        let mel_bands = self.mel_filterbank.apply(self.spectrums.newer());
        let spectral_flux = self.compute_spectral_flux();
        let zero_crossing_rate = Self::compute_zero_crossing_rate(samples);
        let is_transient = self.detect_transient(energy);

        self.prev_energy = energy;

        AudioState {
            mel_bands,
            energy,
            _spectral_flux: spectral_flux,
            _zero_crossing_rate: zero_crossing_rate,
            is_transient,
            timestamp: Instant::now(),
        }
    }

    #[expect(clippy::cast_precision_loss, reason = "HOP_SIZE fits in f64 mantissa")]
    fn compute_energy(samples: &AudioSamples) -> f64 {
        let sum_squares: f64 = samples.iter().map(|s| s * s).sum();
        (sum_squares / HOP_SIZE as f64).sqrt()
    }

    fn compute_spectral_flux(&self) -> f64 {
        self.spectrums
            .newer()
            .iter()
            .zip(self.spectrums.older())
            .map(|(current, previous)| (current - previous).max(0.0))
            .sum()
    }

    #[expect(clippy::cast_precision_loss, reason = "HOP_SIZE fits in f64 mantissa")]
    fn compute_zero_crossing_rate(samples: &AudioSamples) -> f64 {
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0].is_sign_positive() != pair[1].is_sign_positive())
            .count();
        crossings as f64 / HOP_SIZE as f64
    }

    fn detect_transient(&self, energy: f64) -> bool {
        energy - self.prev_energy > TRANSIENT_THRESHOLD
    }
}
