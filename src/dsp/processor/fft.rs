use realfft::num_complex::Complex;
use realfft::{RealToComplex, RealToComplexEven};
use rustfft::FftPlanner;

use super::SPECTRUM_SIZE;
use crate::dsp::{AudioSamples, HOP_SIZE};

/// Real-to-complex FFT with pre-allocated buffers.
pub struct Fft {
    algo: RealToComplexEven<f64>,
    input_buffer: [f64; HOP_SIZE],
    complex_buffer: [Complex<f64>; SPECTRUM_SIZE],
}

impl Fft {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let algo = RealToComplexEven::new(HOP_SIZE, &mut planner);

        Self {
            algo,
            input_buffer: [0.0; HOP_SIZE],
            complex_buffer: [Complex::new(0.0, 0.0); SPECTRUM_SIZE],
        }
    }

    /// Compute the power spectrum (|X[k]|²) of the given samples into `out`.
    ///
    /// Writes `SPECTRUM_SIZE` values. Zero allocation: all buffers are pre-allocated.
    pub fn power_spectrum(&mut self, samples: &AudioSamples, out: &mut [f64; SPECTRUM_SIZE]) {
        self.fft(samples);
        for (p, c) in out.iter_mut().zip(self.complex_buffer.iter()) {
            *p = c.norm_sqr();
        }
    }

    fn fft(&mut self, samples: &AudioSamples) {
        self.input_buffer.copy_from_slice(samples);
        self.algo
            .process_with_scratch(&mut self.input_buffer, &mut self.complex_buffer, &mut [])
            .expect("FFT buffers have correct sizes");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_buffer_size_matches_algo() {
        let fft = Fft::new();
        assert_eq!(fft.input_buffer.len(), fft.algo.len());
    }

    #[test]
    fn complex_buffer_size_matches_algo() {
        let fft = Fft::new();
        assert_eq!(fft.complex_buffer.len(), fft.algo.complex_len());
    }

    #[test]
    fn scratch_not_needed() {
        let fft = Fft::new();
        assert_eq!(0, fft.algo.get_scratch_len());
    }

    #[test]
    fn power_spectrum_runs() {
        let mut fft = Fft::new();
        let samples = [0.0_f64; HOP_SIZE];
        let mut out = [0.0_f64; SPECTRUM_SIZE];
        fft.power_spectrum(&samples, &mut out);
        assert!(out.iter().all(|&v| v == 0.0));
    }
}
