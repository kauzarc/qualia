use realfft::num_complex::Complex;
use realfft::{RealToComplex, RealToComplexEven};
use rustfft::FftPlanner;

use crate::dsp::{AudioSamples, HOP_SIZE};

/// Real-to-complex FFT with pre-allocated buffers.
pub struct Fft {
    algo: RealToComplexEven<f64>,
    input_buffer: Box<[f64]>,
    complex_buffer: Box<[Complex<f64>]>,
    scratch_buffer: Box<[Complex<f64>]>,
    power_buffer: Box<[f64]>,
}

impl Fft {
    pub fn new() -> Self {
        let mut planner = FftPlanner::new();
        let algo = RealToComplexEven::new(HOP_SIZE, &mut planner);
        let spectrum_size = algo.complex_len();

        Self {
            input_buffer: algo.make_input_vec().into_boxed_slice(),
            complex_buffer: algo.make_output_vec().into_boxed_slice(),
            scratch_buffer: algo.make_scratch_vec().into_boxed_slice(),
            power_buffer: vec![0.0; spectrum_size].into_boxed_slice(),
            algo,
        }
    }

    /// Compute the power spectrum (|X[k]|²) of the given samples.
    ///
    /// Returns `HOP_SIZE / 2 + 1` values. Zero allocation: all buffers are pre-allocated.
    pub fn power_spectrum(&mut self, samples: &AudioSamples) -> &[f64] {
        self.fft(samples);
        for (p, c) in self.power_buffer.iter_mut().zip(self.complex_buffer.iter()) {
            *p = c.norm_sqr();
        }
        &self.power_buffer
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
