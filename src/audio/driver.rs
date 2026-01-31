//! Audio capture driver.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use rtrb::Producer;
use tracing::{debug, error, warn};

use super::{AudioDriverError, AudioSamples, HOP_SIZE};

/// Accumulates variable-sized audio chunks into fixed `HOP_SIZE` buffers.
struct HopAccumulator {
    buffer: AudioSamples,
    pos: usize,
}

impl HopAccumulator {
    /// Creates a new accumulator with an empty buffer.
    fn new() -> Self {
        Self {
            buffer: [0.0; HOP_SIZE],
            pos: 0,
        }
    }

    /// Accumulates a sample, returning a complete hop if ready.
    fn accumulate(&mut self, sample: f32) -> Option<AudioSamples> {
        self.buffer[self.pos] = sample;
        self.pos += 1;

        (self.pos == HOP_SIZE).then(|| {
            self.pos = 0;
            self.buffer
        })
    }
}

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
    pub fn try_new(producer: Producer<AudioSamples>) -> Result<Self, AudioDriverError> {
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

    /// Builds the cpal input stream with sample accumulation.
    fn build_stream(
        device: &Device,
        config: &StreamConfig,
        mut producer: Producer<AudioSamples>,
    ) -> Result<Stream, cpal::BuildStreamError> {
        let mut accumulator = HopAccumulator::new();

        device.build_input_stream(
            config,
            move |data: &[f32], _info| {
                for &sample in data {
                    if let Some(hop) = accumulator.accumulate(sample)
                        && producer.push(hop).is_err()
                    {
                        warn!("Sample buffer full, dropping frame");
                    }
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
