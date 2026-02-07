//! Digital signal processing module.
//!
//! Computes audio features from raw samples: FFT, Mel spectrogram, energy,
//! spectral flux, zero-crossing rate, and transient detection.

mod input;
mod orchestrator;
mod processor;
mod state;
mod thread;

pub use state::AudioState;
pub use thread::{DspThread, DspThreadError};

/// Number of samples per audio frame, ~10.7ms at 48kHz.
pub const HOP_SIZE: usize = 512;

/// Audio samples buffer for DSP processing.
pub type AudioSamples = [f64; HOP_SIZE];

/// Number of Mel frequency bands for spectral analysis.
pub const MEL_BANDS: usize = 64;
