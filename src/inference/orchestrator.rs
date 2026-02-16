//! Inference pipeline orchestration.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use rtrb::{Consumer, Producer};
use tracing::{error, info, warn};
use winit::event_loop::EventLoopProxy;

use crate::AppEvent;
use crate::dsp::AudioState;

use super::params::VisualParams;
use super::passthrough::PassthroughModel;
use super::pipe::{InferencePipe, TickResult};

/// Sleep duration when no new input is available.
const IDLE_SLEEP: Duration = Duration::from_millis(1);

/// Orchestrates the inference pipeline: drains input, runs the model,
/// and pushes output.
pub struct InferenceOrchestrator {
    pipe: InferencePipe<PassthroughModel>,
    proxy: EventLoopProxy<AppEvent>,
}

impl InferenceOrchestrator {
    pub fn new(
        state_consumer: Consumer<AudioState>,
        params_producer: Producer<VisualParams>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Self {
        let model = PassthroughModel::new(16);
        let pipe = InferencePipe::new(model, state_consumer, params_producer);

        Self { pipe, proxy }
    }

    /// Runs the core loop until the stop flag is set.
    pub fn run(mut self, stop_flag: &AtomicBool) {
        info!("Inference thread started");

        while !stop_flag.load(Ordering::Relaxed) {
            match self.pipe.tick() {
                TickResult::Produced => {
                    if self
                        .proxy
                        .send_event(AppEvent::VisualParamsProduced)
                        .is_err()
                    {
                        error!("Event loop closed, stopping inference");
                        break;
                    }
                }
                TickResult::NoInput => thread::sleep(IDLE_SLEEP),
                TickResult::BufferFull => warn!("VisualParams buffer full, dropping frame"),
            }
        }

        info!("Inference thread stopped");
    }
}
