use super::MEL_BANDS;

/// Audio features extracted by `DspEngine`, consumed by `Inference`.
#[derive(Clone, Copy)]
pub struct AudioState {
    pub mel_bands: [f64; MEL_BANDS],
    pub energy: f64,
    pub _spectral_flux: f64,
    pub _zero_crossing_rate: f64,
    pub is_transient: bool,
    pub timestamp: u64,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            mel_bands: [0.0; MEL_BANDS],
            energy: 0.0,
            _spectral_flux: 0.0,
            _zero_crossing_rate: 0.0,
            is_transient: false,
            timestamp: 0,
        }
    }
}
