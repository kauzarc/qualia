use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::error;

use crate::audio::AudioSamples;

use super::orchestrator::DspOrchestrator;
use super::state::AudioState;

/// Errors that can occur when initializing `DspThread`.
#[derive(Debug, Error)]
pub enum DspThreadError {
    #[error("Failed to spawn DSP thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Manages the DSP thread lifecycle: spawn and join.
pub struct DspThread {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl DspThread {
    /// Creates and starts the DSP thread.
    pub fn try_new(
        samples_consumer: Consumer<AudioSamples>,
        state_producer: Producer<AudioState>,
    ) -> Result<Self, DspThreadError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();

        let handle = thread::Builder::new().name("dsp".into()).spawn(move || {
            DspOrchestrator::new(samples_consumer, state_producer).run(&thread_stop_flag);
        })?;

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
            error!("DSP thread panicked");
        }
    }
}

impl Drop for DspThread {
    fn drop(&mut self) {
        self.stop();
    }
}
