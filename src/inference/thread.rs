use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::error;
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;
use crate::dsp::AudioState;

use super::orchestrator::InferenceOrchestrator;
use super::params::VisualParams;

/// Errors that can occur when initializing `InferenceThread`.
#[derive(Debug, Error)]
pub enum InferenceThreadError {
    #[error("Failed to spawn inference thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Manages the inference thread lifecycle: spawn and join.
pub struct InferenceThread {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl InferenceThread {
    /// Creates and starts the inference thread.
    pub fn try_new(
        state_consumer: Consumer<AudioState>,
        params_producer: Producer<VisualParams>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<Self, InferenceThreadError> {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();

        let handle = thread::Builder::new()
            .name("inference".into())
            .spawn(move || {
                InferenceOrchestrator::new(state_consumer, params_producer, proxy)
                    .run(&thread_stop_flag);
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
            error!("Inference thread panicked");
        }
    }
}

impl Drop for InferenceThread {
    fn drop(&mut self) {
        self.stop();
    }
}
