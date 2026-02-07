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

/// Number of Mel frequency bands for spectral analysis.
pub const MEL_BANDS: usize = 64;
