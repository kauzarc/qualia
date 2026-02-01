//! Channel infrastructure for pipeline communication.
//!
//! Provides ring buffer connections between pipeline stages with the "drain to latest" strategy.

mod channels;
mod pipe;

pub use channels::Channels;
pub use pipe::{Pipe, Processor};
