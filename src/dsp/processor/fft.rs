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
    scratch_buffer: Box<[Complex<f64>]>,
}

impl Fft {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let algo = RealToComplexEven::new(HOP_SIZE, &mut planner);

        Self {
            scratch_buffer: algo.make_scratch_vec().into_boxed_slice(),
            input_buffer: [0.0; HOP_SIZE],
            complex_buffer: [Complex::new(0.0, 0.0); SPECTRUM_SIZE],
            algo,
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
            .process_with_scratch(
                &mut self.input_buffer,
                &mut self.complex_buffer,
                &mut self.scratch_buffer,
            )
            .expect("FFT buffers have correct sizes");
    }
}
