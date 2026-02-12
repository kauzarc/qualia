mod fft;
mod mel;

use std::time::Instant;

use self::fft::Fft;
use self::mel::MelFilterbank;
use super::state::AudioState;
use super::{AudioSamples, HOP_SIZE, SAMPLE_RATE};

/// Number of FFT frequency bins (`HOP_SIZE / 2 + 1`).
const SPECTRUM_SIZE: usize = HOP_SIZE / 2 + 1;

/// Threshold for transient detection based on energy delta.
const TRANSIENT_THRESHOLD: f64 = 0.1;

/// Handles DSP computations: FFT, Mel spectrogram, and feature extraction.
pub struct DspProcessor {
    prev_energy: f64,
    fft: Fft,
    mel_filterbank: MelFilterbank,
    spectrum_buffer: [f64; SPECTRUM_SIZE],
}

impl DspProcessor {
    pub fn new() -> Self {
        Self {
            prev_energy: 0.0,
            fft: Fft::new(),
            mel_filterbank: MelFilterbank::new(SAMPLE_RATE),
            spectrum_buffer: [0.0; SPECTRUM_SIZE],
        }
    }

    pub fn process(&mut self, samples: &AudioSamples) -> AudioState {
        self.fft.power_spectrum(samples, &mut self.spectrum_buffer);
        let energy = self.compute_energy(samples);
        let mel_bands = self.mel_filterbank.apply(&self.spectrum_buffer);
        let spectral_flux = self.compute_spectral_flux(samples);
        let zero_crossing_rate = self.compute_zero_crossing_rate(samples);
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
}

#[expect(clippy::unused_self, reason = "will use self for stateful processing")]
impl DspProcessor {
    #[expect(clippy::cast_precision_loss, reason = "MEL_BANDS fits in f64 mantissa")]
    fn compute_energy(&self, samples: &AudioSamples) -> f64 {
        let sum_squares: f64 = samples.iter().map(|s| s * s).sum();
        (sum_squares / HOP_SIZE as f64).sqrt()
    }

    fn compute_spectral_flux(&self, _samples: &AudioSamples) -> f64 {
        // TODO: compute spectral flux from FFT magnitude difference
        0.0
    }

    fn compute_zero_crossing_rate(&self, _samples: &AudioSamples) -> f64 {
        // TODO: count zero crossings / sample count
        0.0
    }

    fn detect_transient(&self, energy: f64) -> bool {
        energy - self.prev_energy > TRANSIENT_THRESHOLD
    }
}
