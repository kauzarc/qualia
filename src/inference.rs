//! Neural network inference module.
//!
//! Runs the trained model to transform `AudioState` into `VisualParams`.

mod engine;
mod model;
mod params;
mod passthrough;
mod processor;

pub use engine::{Inference, InferenceError};
pub use params::VisualParams;
