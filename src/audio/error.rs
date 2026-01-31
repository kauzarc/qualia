//! Audio driver error types.

use thiserror::Error;

/// Errors that can occur when initializing the audio driver.
#[derive(Debug, Error)]
pub enum AudioDriverError {
    /// No audio input device available.
    #[error("No input device available")]
    NoInputDevice,

    /// Failed to get default input configuration.
    #[error("Failed to get default input config: {0}")]
    DefaultConfig(#[from] cpal::DefaultStreamConfigError),

    /// Failed to build input stream.
    #[error("Failed to build input stream: {0}")]
    BuildStream(#[from] cpal::BuildStreamError),

    /// Failed to start the audio stream.
    #[error("Failed to start stream: {0}")]
    PlayStream(#[from] cpal::PlayStreamError),
}
