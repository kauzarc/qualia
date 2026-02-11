use std::array;

use super::SPECTRUM_SIZE;
use crate::dsp::{HOP_SIZE, MEL_BANDS};

/// Per-band metadata indexing into `SparseMelFilterbank::weights`.
#[derive(Clone, Copy)]
struct BandRange {
    /// First non-zero bin index in the power spectrum.
    spectrum_start: usize,
    /// Offset into the flat weights array.
    weight_offset: usize,
    /// Number of non-zero weights for this band.
    len: usize,
}

/// Sparse mel filterbank: all non-zero triangular weights packed into a single
/// contiguous array, with per-band metadata for offset and length.
///
/// Applying this filterbank touches only the non-zero weights and their
/// corresponding spectrum bins — both as contiguous slices.
pub struct SparseMelFilterbank {
    bands: [BandRange; MEL_BANDS],
    weights: Box<[f64]>,
}

impl SparseMelFilterbank {
    /// Apply the filterbank to a power spectrum, writing mel band energies into `out`.
    pub fn apply(&self, spectrum: &[f64; SPECTRUM_SIZE], out: &mut [f64; MEL_BANDS]) {
        self.bands.iter().zip(out.iter_mut()).for_each(
            |(
                &BandRange {
                    spectrum_start,
                    weight_offset,
                    len,
                },
                out,
            )| {
                let weight = &self.weights[weight_offset..weight_offset + len];
                let spec = &spectrum[spectrum_start..spectrum_start + len];
                *out = weight
                    .iter()
                    .zip(spec)
                    .map(|(weight, spec)| weight * spec)
                    .sum();
            },
        );
    }
}

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
        let bins = centers_hz.map(|hz| Self::hz_to_fft_bin(hz, sample_rate));

        let weights = array::from_fn(|band| {
            let left = bins[band];
            let center = bins[band + 1];
            let right = bins[band + 2];

            array::from_fn(|bin| Self::triangle_weight(bin, left, center, right))
        });

        Self { weights }
    }

    /// Extract a sparse filterbank from the dense weight matrix.
    ///
    /// Packs all non-zero weights into a single contiguous array and records
    /// per-band offsets for zero-overhead apply.
    pub fn into_sparse(self) -> SparseMelFilterbank {
        let mut weights = Vec::new();
        let bands = self.weights.map(|row| {
            if let Some(first) = row.iter().position(|&w| w > 0.0) {
                let last = row
                    .iter()
                    .rposition(|&w| w > 0.0)
                    .expect("At least one non zero");

                let current_row_weights = &row[first..=last];
                let weight_offset = weights.len();

                weights.extend_from_slice(current_row_weights);

                BandRange {
                    spectrum_start: first,
                    weight_offset,
                    len: current_row_weights.len(),
                }
            } else {
                BandRange {
                    spectrum_start: 0,
                    weight_offset: 0,
                    len: 0,
                }
            }
        });

        SparseMelFilterbank {
            bands,
            weights: weights.into_boxed_slice(),
        }
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
        [0.0, 100.0, 440.0, 1000.0, 8000.0, 24000.0]
            .iter()
            .for_each(|&hz| {
                let round_tripped =
                    DenseMelFilterbank::mel_to_hz(DenseMelFilterbank::hz_to_mel(hz));
                assert!(
                    (round_tripped - hz).abs() < 1e-6,
                    "round-trip failed for {hz} Hz: got {round_tripped}"
                );
            });
    }

    #[test]
    fn center_frequencies_monotonically_increasing() {
        let centers = DenseMelFilterbank::mel_center_frequencies(SAMPLE_RATE);
        centers.windows(2).for_each(|pair| {
            assert!(
                pair[1] > pair[0],
                "centers not monotonic: {} >= {}",
                pair[0],
                pair[1]
            );
        });
    }

    #[test]
    fn center_frequencies_span_range() {
        let centers = DenseMelFilterbank::mel_center_frequencies(SAMPLE_RATE);
        let nyquist = SAMPLE_RATE as f64 / 2.0;
        assert!(
            (centers[0] - DenseMelFilterbank::F_MIN).abs() < 1.0,
            "first center should be near F_MIN"
        );
        assert!((centers[MEL_BANDS + 1] - nyquist).abs() < 1.0);
    }

    #[test]
    fn every_band_has_nonzero_weights() {
        let fb = DenseMelFilterbank::new(SAMPLE_RATE);
        fb.weights.iter().enumerate().for_each(|(i, band)| {
            let sum: f64 = band.iter().sum();
            assert!(sum > 0.0, "band {i} has no nonzero weights");
        });
    }

    #[test]
    fn zero_spectrum_gives_zero_bands() {
        let fb = DenseMelFilterbank::new(SAMPLE_RATE);
        let spectrum = [0.0_f64; SPECTRUM_SIZE];
        fb.weights.iter().for_each(|band| {
            let result: f64 = band
                .iter()
                .zip(spectrum.iter())
                .map(|(weight, spec)| weight * spec)
                .sum();
            assert!(result.abs() < f64::EPSILON);
        });
    }

    #[test]
    fn sparse_fewer_weights_than_dense() {
        let sparse = DenseMelFilterbank::new(SAMPLE_RATE).into_sparse();
        assert!(sparse.weights.len() < MEL_BANDS * SPECTRUM_SIZE);
    }

    #[test]
    fn sparse_every_band_has_nonzero_weights() {
        let sparse = DenseMelFilterbank::new(SAMPLE_RATE).into_sparse();
        assert!(sparse.bands.iter().all(|band| band.len > 0));
    }

    #[test]
    fn sparse_zero_spectrum_gives_zero_bands() {
        let sparse = DenseMelFilterbank::new(SAMPLE_RATE).into_sparse();
        let spectrum = [0.0_f64; SPECTRUM_SIZE];
        let mut out = [f64::NAN; MEL_BANDS];
        sparse.apply(&spectrum, &mut out);
        assert!(out.iter().all(|val| val.abs() < f64::EPSILON));
    }

    #[test]
    fn sparse_matches_dense() {
        let dense = DenseMelFilterbank::new(SAMPLE_RATE);
        let spectrum: [f64; SPECTRUM_SIZE] = array::from_fn(|i| (i as f64 * 0.1).sin().powi(2));

        let dense_result: [f64; MEL_BANDS] = array::from_fn(|band| {
            dense.weights[band]
                .iter()
                .zip(spectrum.iter())
                .map(|(weight, spec)| weight * spec)
                .sum()
        });

        let sparse = dense.into_sparse();
        let mut sparse_result = [0.0_f64; MEL_BANDS];
        sparse.apply(&spectrum, &mut sparse_result);

        assert!(
            dense_result
                .iter()
                .zip(sparse_result.iter())
                .all(|(dense, sparse)| (dense - sparse).abs() < f64::EPSILON)
        );
    }
}
