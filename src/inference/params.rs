/// Maximum number of visual control parameters.
pub const MAX_ACTIONS: usize = 64;

/// Normalized value in [0.0, 1.0] for shader control.
#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
pub struct ControlVoltage(f32);

impl ControlVoltage {
    /// Creates a new `ControlVoltage` if value is in [0.0, 1.0].
    pub fn _new(value: f32) -> Option<Self> {
        (0.0..=1.0).contains(&value).then_some(Self(value))
    }

    /// Creates a new `ControlVoltage`, clamping to [0.0, 1.0].
    pub fn clamped(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Returns the inner value.
    pub fn _get(self) -> f32 {
        self.0
    }
}

/// Visual parameters produced by `Inference`, consumed by `Display`.
#[derive(Clone, Copy)]
pub struct VisualParams {
    pub _actions: [ControlVoltage; MAX_ACTIONS],
    pub _num_actions: usize,
    pub _is_transient: bool,
    pub _timestamp: u64,
}

impl Default for VisualParams {
    fn default() -> Self {
        Self {
            _actions: [ControlVoltage::default(); MAX_ACTIONS],
            _num_actions: 16,
            _is_transient: false,
            _timestamp: 0,
        }
    }
}
