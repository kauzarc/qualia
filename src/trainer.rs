use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::thread::{self, JoinHandle};

use thiserror::Error;
use tracing::{error, info};

/// Feedback sent from Display to Trainer.
#[derive(Clone, Copy, Debug)]
pub struct Feedback {
    pub value: f32, // [-1.0, 1.0] per spec
    pub timestamp: u64,
}

#[derive(Debug, Error)]
pub enum TrainerError {
    #[error("Failed to spawn trainer thread: {0}")]
    SpawnThread(#[from] io::Error),
}

pub struct Trainer {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl Trainer {
    pub fn try_new(_feedback_receiver: Receiver<Feedback>) -> Result<Self, TrainerError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("trainer".into())
                .spawn(move || {
                    info!("Trainer thread started");

                    while !stop_flag.load(Ordering::Relaxed) {
                        // TODO: receive feedback, update replay buffer, train model
                        thread::sleep(std::time::Duration::from_millis(100));
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
