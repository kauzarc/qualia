//! Audio capture driver.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use rtrb::Producer;
use tracing::{debug, error, warn};

use super::AudioDriverError;

/// Hard real-time audio capture driver.
///
/// Captures raw audio samples via cpal and pushes them to the DSP engine
/// via a lock-free ring buffer. cpal automatically sets real-time thread
/// priority on supported platforms (ALSA, WASAPI).
pub struct AudioDriver {
    _stream: Stream,
}

impl AudioDriver {
    /// Creates and starts the audio driver.
    pub fn try_new(producer: Producer<f64>) -> Result<Self, AudioDriverError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioDriverError::NoInputDevice)?;

        let config = device.default_input_config()?;
        debug!("Audio input config: {:?}", config);

        let stream = Self::build_stream(&device, &config.into(), producer)?;
        stream.play()?;

        Ok(Self { _stream: stream })
    }

    /// Builds the cpal input stream with bulk sample writing.
    ///
    /// Captures as f32 (hardware native) and converts to f64 for processing.
    fn build_stream(
        device: &Device,
        config: &StreamConfig,
        mut producer: Producer<f64>,
    ) -> Result<Stream, cpal::BuildStreamError> {
        device.build_input_stream(
            config,
            move |data: &[f32], _info| {
                if let Ok(chunk) = producer.write_chunk_uninit(data.len()) {
                    chunk.fill_from_iter(data.iter().map(|&s| f64::from(s)));
                } else {
                    warn!("Sample buffer full, dropping {} samples", data.len());
                }
            },
            |err| {
                error!("Audio stream error: {}", err);
                todo!("Handle audio stream error");
            },
            None,
        )
    }
}
