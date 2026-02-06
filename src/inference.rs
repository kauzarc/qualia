//! Neural network inference module.
//!
//! Runs the trained model to transform `AudioState` into `VisualParams`.

mod engine;
mod model;
mod params;
mod passthrough;
mod pipe;

pub use engine::{Inference, InferenceError};
pub use params::{ControlVoltage, VisualParams};
