//! Audio driver error types.

use cpal::StreamError;
use thiserror::Error;

/// Fatal runtime errors from the audio stream.
#[derive(Debug, Clone, Error)]
pub enum AudioStreamFatalError {
    /// The audio device was disconnected.
    #[error("Audio device lost")]
    DeviceLost,

    /// The stream configuration is no longer valid.
    #[error("Stream invalidated")]
    StreamInvalidated,
}

impl TryFrom<StreamError> for AudioStreamFatalError {
    type Error = StreamError;

    fn try_from(err: StreamError) -> Result<Self, Self::Error> {
        match err {
            StreamError::DeviceNotAvailable => Ok(Self::DeviceLost),
            StreamError::StreamInvalidated => Ok(Self::StreamInvalidated),
            other => Err(other),
        }
    }
}

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
