//! Neural network inference module.
//!
//! Runs the trained model to transform `AudioState` into `VisualParams`.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{error, info};

use crate::dsp::AudioState;

/// Maximum number of visual control parameters.
pub const MAX_ACTIONS: usize = 64;

/// Normalized value in [0.0, 1.0] for shader control.
#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub struct ControlVoltage(f32);

#[allow(dead_code)]
impl ControlVoltage {
    /// Creates a new `ControlVoltage` if value is in [0.0, 1.0].
    pub fn new(value: f32) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the inner value.
    pub fn get(self) -> f32 {
        self.0
    }
}

/// Visual parameters produced by `Inference`, consumed by `Display`.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct VisualParams {
    pub actions: [ControlVoltage; MAX_ACTIONS],
    pub num_actions: usize,
    pub is_transient: bool,
    pub timestamp: u64,
}

impl Default for VisualParams {
    fn default() -> Self {
        Self {
            actions: [ControlVoltage::default(); MAX_ACTIONS],
            num_actions: 16,
            is_transient: false,
            timestamp: 0,
        }
    }
}

/// Errors that can occur when initializing `Inference`.
#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Failed to spawn inference thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Inference thread running the neural network model.
///
/// Consumes `AudioState` and produces `VisualParams` for the display.
pub struct Inference {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl Inference {
    /// Creates and starts the inference thread.
    pub fn try_new(
        _state_consumer: Consumer<AudioState>,
        _params_producer: Producer<VisualParams>,
    ) -> Result<Self, InferenceError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("inference".into())
                .spawn(move || {
                    info!("Inference thread started");

                    while !stop_flag.load(Ordering::Relaxed) {
                        // TODO: consume AudioState, run model, produce VisualParams
                        thread::sleep(std::time::Duration::from_millis(16));
                    }

                    info!("Inference thread stopped");
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
            error!("Inference thread panicked");
        }
    }
}

impl Drop for Inference {
    fn drop(&mut self) {
        self.stop();
    }
}
