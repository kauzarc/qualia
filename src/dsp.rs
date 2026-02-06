//! Digital signal processing module.
//!
//! Computes audio features from raw samples: FFT, Mel spectrogram, energy,
//! spectral flux, zero-crossing rate, and transient detection.

mod engine;
mod pipe;
mod processor;
mod state;

pub use engine::{DspEngine, DspEngineError};
pub use state::AudioState;

/// Number of Mel frequency bands for spectral analysis.
pub const MEL_BANDS: usize = 64;
