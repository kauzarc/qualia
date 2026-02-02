//! Audio capture module.
//!
//! Handles raw audio capture via cpal callback at hard real-time priority.
//! Zero-allocation in the audio callback to prevent glitches.

mod driver;
mod error;

pub use driver::AudioDriver;
pub use error::AudioDriverError;

/// Number of samples per audio buffer, ~10.7ms at 48kHz.
pub const HOP_SIZE: usize = 512;

/// Audio samples buffer sent from `AudioDriver` to `DspEngine`.
pub type AudioSamples = [f64; HOP_SIZE];
