//! Audio capture module.
//!
//! Handles raw audio capture via cpal callback at hard real-time priority.
//! Zero-allocation in the audio callback to prevent glitches.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::Producer;
use thiserror::Error;
use tracing::{error, info};

/// Number of samples per audio buffer, ~10.7ms at 48kHz.
pub const HOP_SIZE: usize = 512;

/// Audio samples buffer sent from `AudioDriver` to `DspEngine`.
pub type AudioSamples = [f32; HOP_SIZE];

/// Errors that can occur when initializing the audio driver.
#[derive(Debug, Error)]
pub enum AudioDriverError {
    #[error("Failed to spawn audio driver thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Hard real-time audio capture thread.
///
/// Captures raw audio samples and pushes them to the DSP engine via a lock-free
/// ring buffer. Runs at the highest priority to prevent audio glitches.
pub struct AudioDriver {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl AudioDriver {
    /// Creates and starts the audio driver thread.
    pub fn try_new(_samples_tx: Producer<AudioSamples>) -> Result<Self, AudioDriverError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("audio-driver".into())
                .spawn(move || {
                    info!("Audio driver thread started");

                    while !stop_flag.load(Ordering::Relaxed) {
                        // TODO: cpal capture or synthetic signal, push to samples_tx
                        thread::sleep(std::time::Duration::from_millis(11));
                    }

                    info!("Audio driver thread stopped");
                })?
        };

        Ok(Self {
            handle: Some(handle),
            stop_flag,
        })
    }

    fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            error!("Audio driver thread panicked");
        }
    }
}

impl Drop for AudioDriver {
    fn drop(&mut self) {
        self.stop();
    }
}
