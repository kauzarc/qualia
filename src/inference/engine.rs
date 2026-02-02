use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{error, info, warn};
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;
use crate::channel::Pipe;
use crate::dsp::AudioState;

use super::params::VisualParams;
use super::passthrough::PassthroughModel;
use super::processor::InferenceProcessor;

/// Errors that can occur when initializing `Inference`.
#[derive(Debug, Error)]
pub enum InferenceError {
    #[error("Failed to spawn inference thread: {0}")]
    SpawnThread(#[from] io::Error),
}

/// Inference thread running the neural network model.
pub struct Inference {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl Inference {
    /// Creates and starts the inference thread.
    pub fn try_new(
        state_consumer: Consumer<AudioState>,
        params_producer: Producer<VisualParams>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Result<Self, InferenceError> {
        let stop_flag = Arc::new(AtomicBool::new(false));

        let handle = {
            let stop_flag = stop_flag.clone();
            thread::Builder::new()
                .name("inference".into())
                .spawn(move || Self::run(&stop_flag, state_consumer, params_producer, proxy))?
        };

        Ok(Self {
            handle: Some(handle),
            stop_flag,
        })
    }

    fn run(
        stop_flag: &AtomicBool,
        state_consumer: Consumer<AudioState>,
        params_producer: Producer<VisualParams>,
        proxy: EventLoopProxy<AppEvent>,
    ) {
        info!("Inference thread started");

        let model = PassthroughModel::new(16);
        let processor = InferenceProcessor::new(model);
        let mut pipe = Pipe::new(processor, state_consumer, params_producer);

        while !stop_flag.load(Ordering::Relaxed) {
            match pipe.tick() {
                Ok(true) => {
                    if proxy.send_event(AppEvent::VisualParamsProduced).is_err() {
                        error!("Event loop closed, stopping inference");
                        break;
                    }
                }
                Ok(false) => thread::sleep(std::time::Duration::from_millis(1)),
                Err(_) => warn!("VisualParams buffer full, dropping frame"),
            }
        }

        info!("Inference thread stopped");
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
