use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{error, info, warn};

use crate::audio::AudioSamples;
use crate::channel::Pipe;

use super::processor::DspProcessor;
use super::state::AudioState;

/// Errors that can occur when initializing `DspEngine`.
#[derive(Debug, Error)]
pub enum DspEngineError {
    #[error("Failed to spawn DSP engine thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// DSP analysis thread.
///
/// Consumes raw audio samples and produces `AudioState` with extracted features.
pub struct DspEngine {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl DspEngine {
    /// Creates and starts the DSP engine thread.
    pub fn try_new(
        samples_consumer: Consumer<AudioSamples>,
        state_producer: Producer<AudioState>,
    ) -> Result<Self, DspEngineError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("dsp-engine".into())
                .spawn(move || Self::run(&stop_flag, samples_consumer, state_producer))?
        };

        Ok(Self {
            handle: Some(handle),
            stop_flag,
        })
    }

    fn run(
        stop_flag: &AtomicBool,
        samples_consumer: Consumer<AudioSamples>,
        state_producer: Producer<AudioState>,
    ) {
        info!("DSP engine thread started");

        let mut pipe = Pipe::new(DspProcessor::new(), samples_consumer, state_producer);

        while !stop_flag.load(Ordering::Relaxed) {
            match pipe.tick() {
                Ok(true) => {}
                Ok(false) => thread::sleep(std::time::Duration::from_millis(1)),
                Err(_) => warn!("AudioState buffer full, dropping frame"),
            }
        }

        info!("DSP engine thread stopped");
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
