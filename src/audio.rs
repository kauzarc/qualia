//! Audio capture module.
//!
//! Handles raw audio capture via cpal callback at hard real-time priority.
//! Zero-allocation in the audio callback to prevent glitches.

mod driver;
mod error;

pub use driver::AudioDriver;
pub use error::AudioDriverError;
