use std::array;

use super::SPECTRUM_SIZE;
use crate::dsp::{HOP_SIZE, MEL_BANDS};

/// Dense mel filterbank matrix for converting a linear power spectrum to mel bands.
pub struct DenseMelFilterbank {
    weights: [[f64; SPECTRUM_SIZE]; MEL_BANDS],
}

impl DenseMelFilterbank {
    /// Build a mel filterbank for the given sample rate.
    ///
    /// Uses fractional bin positions so that narrow low-frequency bands
    /// still produce correct triangular weights via interpolation.
    pub fn new(sample_rate: u64) -> Self {
        let centers_hz = Self::mel_center_frequencies(sample_rate);

        let weights = array::from_fn(|band| {
            let left = Self::hz_to_fft_bin(centers_hz[band], sample_rate);
            let center = Self::hz_to_fft_bin(centers_hz[band + 1], sample_rate);
            let right = Self::hz_to_fft_bin(centers_hz[band + 2], sample_rate);

            array::from_fn(|bin| Self::triangle_weight(bin, left, center, right))
        });

        Self { weights }
    }

    /// Minimum frequency for the mel filterbank (Hz).
    const F_MIN: f64 = 20.0;

    /// O'Shaughnessy mel scale break frequency (Hz).
    const MEL_BREAK_HZ: f64 = 700.0;

    /// O'Shaughnessy mel scale log step.
    const MEL_LOG_STEP: f64 = 2595.0;

    fn hz_to_mel(hz: f64) -> f64 {
        Self::MEL_LOG_STEP * (1.0 + hz / Self::MEL_BREAK_HZ).log10()
    }

    fn mel_to_hz(mel: f64) -> f64 {
        Self::MEL_BREAK_HZ * (10.0_f64.powf(mel / Self::MEL_LOG_STEP) - 1.0)
    }

    /// Return `MEL_BANDS + 2` center frequencies in Hz, evenly spaced on the mel scale
    /// between `F_MIN` and Nyquist.
    #[expect(
        clippy::cast_precision_loss,
        reason = "sample rate and indices fit in f64"
    )]
    fn mel_center_frequencies(sample_rate: u64) -> [f64; MEL_BANDS + 2] {
        let mel_min = Self::hz_to_mel(Self::F_MIN);
        let mel_max = Self::hz_to_mel(sample_rate as f64 / 2.0);
        let num_points = MEL_BANDS + 2;

        array::from_fn(|i| {
            let t = i as f64 / (num_points - 1) as f64;
            Self::mel_to_hz(mel_min + (mel_max - mel_min) * t)
        })
    }

    /// Map a frequency in Hz to a fractional FFT bin position.
    #[expect(
        clippy::cast_precision_loss,
        reason = "HOP_SIZE and sample rates fit in f64"
    )]
    fn hz_to_fft_bin(hz: f64, sample_rate: u64) -> f64 {
        hz * HOP_SIZE as f64 / sample_rate as f64
    }

    /// Triangular window weight for a fractional bin position.
    ///
    /// Returns the height of a triangle with base `[left, right]` and peak at `center`,
    /// evaluated at position `bin`.
    #[expect(clippy::cast_precision_loss, reason = "bin index fits in f64")]
    fn triangle_weight(bin: usize, left: f64, center: f64, right: f64) -> f64 {
        let b = bin as f64;
        if b >= left && b < center && center > left {
            (b - left) / (center - left)
        } else if b >= center && b <= right && right > center {
            (right - b) / (right - center)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u64 = 48_000;

    #[test]
    fn hz_mel_round_trip() {
        for &hz in &[0.0, 100.0, 440.0, 1000.0, 8000.0, 24000.0] {
            let round_tripped = DenseMelFilterbank::mel_to_hz(DenseMelFilterbank::hz_to_mel(hz));
            assert!(
                (round_tripped - hz).abs() < 1e-6,
                "round-trip failed for {hz} Hz: got {round_tripped}"
            );
        }
    }

    #[test]
    fn center_frequencies_monotonically_increasing() {
        let centers = DenseMelFilterbank::mel_center_frequencies(SAMPLE_RATE);
        for pair in centers.windows(2) {
            assert!(
                pair[1] > pair[0],
                "centers not monotonic: {} >= {}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn center_frequencies_span_range() {
        let centers = DenseMelFilterbank::mel_center_frequencies(SAMPLE_RATE);
        assert!(
            (centers[0] - DenseMelFilterbank::F_MIN).abs() < 1.0,
            "first center should be near F_MIN"
        );
        let nyquist = SAMPLE_RATE as f64 / 2.0;
        assert!((centers[MEL_BANDS + 1] - nyquist).abs() < 1.0);
    }

    #[test]
    fn every_band_has_nonzero_weights() {
        let fb = DenseMelFilterbank::new(SAMPLE_RATE);
        for (i, band) in fb.weights.iter().enumerate() {
            let sum: f64 = band.iter().sum();
            assert!(sum > 0.0, "band {i} has no nonzero weights");
        }
    }

    #[test]
    fn zero_spectrum_gives_zero_bands() {
        let fb = DenseMelFilterbank::new(SAMPLE_RATE);
        let spectrum = [0.0_f64; SPECTRUM_SIZE];
        for band in &fb.weights {
            let result: f64 = band.iter().zip(spectrum.iter()).map(|(w, s)| w * s).sum();
            assert!((result - 0.0).abs() < f64::EPSILON);
        }
    }
}
