//! Neural network inference module.
//!
//! Runs the trained model to transform `AudioState` into `VisualParams`.

mod input;
mod model;
mod orchestrator;
mod params;
mod passthrough;
mod thread;

pub use params::{ControlVoltage, VisualParams};
pub use thread::{InferenceThread, InferenceThreadError};
