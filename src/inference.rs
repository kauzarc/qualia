//! Neural network inference module.
//!
//! Runs the trained model to transform `AudioState` into `VisualParams`.

mod model;
mod orchestrator;
mod params;
mod passthrough;
mod pipe;
mod thread;

pub use params::{ControlVoltage, VisualParams};
pub use thread::{InferenceThread, InferenceThreadError};
