use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{error, info};

use crate::dsp::AudioState;

pub const MAX_ACTIONS: usize = 64;

#[derive(Clone, Copy, Default)]
pub struct ControlVoltage(f32);

impl ControlVoltage {
    pub fn new(value: f32) -> Option<Self> {
        if (0.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn get(self) -> f32 {
        self.0
    }
}

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

#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Failed to spawn inference thread: {0}")]
    SpawnThread(#[from] io::Error),
}

pub struct Inference {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl Inference {
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
