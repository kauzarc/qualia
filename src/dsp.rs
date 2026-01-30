use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{error, info};

use crate::audio::AudioSamples;

pub const MEL_BANDS: usize = 64;

/// Audio state produced by DSP, consumed by Inference.
#[derive(Clone, Copy)]
pub struct AudioState {
    pub mel_bands: [f32; MEL_BANDS],
    pub rms: f32,
    pub spectral_flux: f32,
    pub zcr: f32,
    pub is_transient: bool,
    pub timestamp: u64,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            mel_bands: [0.0; MEL_BANDS],
            rms: 0.0,
            spectral_flux: 0.0,
            zcr: 0.0,
            is_transient: false,
            timestamp: 0,
        }
    }
}

#[derive(Debug, Error)]
pub enum DspEngineError {
    #[error("Failed to spawn DSP engine thread: {0}")]
    SpawnThread(#[from] io::Error),
}

pub struct DspEngine {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl DspEngine {
    pub fn try_new(
        _samples_consumer: Consumer<AudioSamples>,
        _state_producer: Producer<AudioState>,
    ) -> Result<Self, DspEngineError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("dsp-engine".into())
                .spawn(move || {
                    info!("DSP engine thread started");

                    while !stop_flag.load(Ordering::Relaxed) {
                        // TODO: consume samples, compute FFT/Mel/features, produce AudioState
                        thread::sleep(std::time::Duration::from_millis(11));
                    }

                    info!("DSP engine thread stopped");
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
            error!("DSP engine thread panicked");
        }
    }
}

impl Drop for DspEngine {
    fn drop(&mut self) {
        self.stop();
    }
}
