//! Online learning module.
//!
//! Receives user feedback, maintains a replay buffer, and trains the model
//! asynchronously without blocking the real-time audio/visual pipeline.

use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use thiserror::Error;
use tracing::{error, info};

/// User feedback value in [-1.0, 1.0].
#[derive(Clone, Copy, Debug)]
pub struct Reward(f64);

impl Reward {
    /// Creates a new `Reward` if value is in [-1.0, 1.0].
    pub fn new(value: f64) -> Option<Self> {
        if (-1.0..=1.0).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    /// Returns the inner value.
    #[expect(dead_code, reason = "will be used by trainer")]
    pub fn get(self) -> f64 {
        self.0
    }
}

/// User feedback event sent from `Display` to `Trainer`.
#[expect(dead_code, reason = "fields will be used by trainer")]
#[derive(Clone, Copy, Debug)]
pub struct Feedback {
    pub reward: Reward,
    pub timestamp: u64,
}

/// Errors that can occur when initializing `Trainer`.
#[derive(Debug, Error)]
pub enum TrainerError {
    #[error("Failed to spawn trainer thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Background training thread.
///
/// Receives feedback, assigns credit to past actions, and updates the model.
pub struct Trainer {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl Trainer {
    /// Creates and starts the trainer thread.
    pub fn try_new(feedback_receiver: Receiver<Feedback>) -> Result<Self, TrainerError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("trainer".into())
                .spawn(move || {
                    info!("Trainer thread started");

                    while !stop_flag.load(Ordering::Relaxed) {
                        match feedback_receiver.recv_timeout(Duration::from_millis(100)) {
                            Ok(feedback) => {
                                info!("Received feedback: {:?}", feedback);
                            }
                            Err(RecvTimeoutError::Timeout) => {}
                            Err(RecvTimeoutError::Disconnected) => {
                                info!("Feedback channel disconnected");
                                break;
                            }
                        }
                    }

                    info!("Trainer thread stopped");
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
            error!("Trainer thread panicked");
        }
    }
}

impl Drop for Trainer {
    fn drop(&mut self) {
        self.stop();
    }
}
