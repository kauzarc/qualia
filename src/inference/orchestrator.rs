//! Inference pipeline orchestration.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use rtrb::{Consumer, Producer};
use thiserror::Error;
use tracing::{info, warn};
use winit::event_loop::{EventLoopClosed, EventLoopProxy};

use crate::AppEvent;
use crate::dsp::AudioState;

use super::input::AudioStateInput;
use super::model::InferenceModel;
use super::params::VisualParams;
use super::passthrough::PassthroughModel;

/// Sleep duration when no new input is available.
const IDLE_SLEEP: Duration = Duration::from_millis(1);

/// Error returned when the inference loop terminates unexpectedly.
#[derive(Debug, Error)]
#[error("Event loop closed unexpectedly")]
pub struct InferenceRunError(#[from] EventLoopClosed<AppEvent>);

/// Orchestrates the inference pipeline: drains input, runs the model,
/// and pushes output.
pub struct InferenceOrchestrator {
    input: AudioStateInput,
    model: PassthroughModel,
    output: Producer<VisualParams>,
    proxy: EventLoopProxy<AppEvent>,
}

impl InferenceOrchestrator {
    pub fn new(
        state_consumer: Consumer<AudioState>,
        params_producer: Producer<VisualParams>,
        proxy: EventLoopProxy<AppEvent>,
    ) -> Self {
        Self {
            input: AudioStateInput::new(state_consumer),
            model: PassthroughModel::new(16),
            output: params_producer,
            proxy,
        }
    }

    /// Runs the core loop until the stop flag is set.
    ///
    /// Returns `Err` if the event loop closes unexpectedly.
    pub fn run(mut self, stop_flag: &AtomicBool) -> Result<(), InferenceRunError> {
        info!("Inference thread started");

        while !stop_flag.load(Ordering::Relaxed) {
            if let Some(state) = self.input.drain_to_latest() {
                let params = self.model.infer(&state);

                if self.output.push(params).is_err() {
                    warn!("VisualParams buffer full, dropping frame");
                } else {
                    self.proxy.send_event(AppEvent::VisualParamsProduced)?;
                }
            } else {
                thread::sleep(IDLE_SLEEP);
            }
        }

        Ok(())
    }
}
