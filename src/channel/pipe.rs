use std::iter;

use rtrb::{Consumer, Producer};

/// Stateful transformation between pipeline stages.
pub trait Processor {
    type Input;
    type Output;

    fn process(&mut self, input: Self::Input) -> Self::Output;
}

/// Connects two ring buffers, transforming input to output.
///
/// Uses "drain to latest" strategy: discards stale inputs, processes only the freshest.
pub struct Pipe<P: Processor> {
    processor: P,
    input: Consumer<P::Input>,
    output: Producer<P::Output>,
}

impl<P: Processor> Pipe<P> {
    pub fn new(processor: P, input: Consumer<P::Input>, output: Producer<P::Output>) -> Self {
        Self {
            processor,
            input,
            output,
        }
    }

    /// Processes the latest available input.
    ///
    /// Returns:
    /// - `Ok(true)` if processed and pushed
    /// - `Ok(false)` if no input available
    /// - `Err(output)` if processed but output buffer full
    pub fn tick(&mut self) -> Result<bool, P::Output> {
        let Some(input) = self.drain_to_latest() else {
            return Ok(false);
        };

        let output = self.processor.process(input);

        self.output
            .push(output)
            .map(|()| true)
            .map_err(|rtrb::PushError::Full(v)| v)
    }

    fn drain_to_latest(&mut self) -> Option<P::Input> {
        iter::from_fn(|| self.input.pop().ok()).last()
    }
}
