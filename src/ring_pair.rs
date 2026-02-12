//! Fixed-size circular buffer of exactly 2 elements.

/// Circular buffer holding exactly 2 elements with O(1) push.
pub struct RingPair<T> {
    buffer: [T; 2],
    newest: usize,
}

impl<T: Default> Default for RingPair<T> {
    fn default() -> Self {
        Self {
            buffer: [T::default(), T::default()],
            newest: 0,
        }
    }
}

impl<T: Copy> RingPair<T> {
    /// Creates a new `RingPair` with both slots initialized to the given value.
    pub fn new(initial: T) -> Self {
        Self {
            buffer: [initial; 2],
            newest: 0,
        }
    }
}

impl<T> RingPair<T> {
    /// Pushes a new value, making it the newest.
    pub fn push(&mut self, value: T) {
        self.newest = 1 - self.newest;
        self.buffer[self.newest] = value;
    }

    /// Advances to the next slot and returns a mutable reference to it,
    /// allowing in-place initialization without copying.
    pub fn push_with<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        self.newest = 1 - self.newest;
        f(&mut self.buffer[self.newest]);
    }

    /// Returns a reference to the newer element.
    pub fn newer(&self) -> &T {
        &self.buffer[self.newest]
    }

    /// Returns a reference to the older element.
    pub fn older(&self) -> &T {
        &self.buffer[1 - self.newest]
    }
}
