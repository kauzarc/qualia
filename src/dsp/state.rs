use super::MEL_BANDS;

/// Audio features extracted by `DspEngine`, consumed by `Inference`.
#[derive(Clone, Copy)]
pub struct AudioState {
    pub mel_bands: [f32; MEL_BANDS],
    pub energy: f32,
    pub spectral_flux: f32,
    pub zero_crossing_rate: f32,
    pub is_transient: bool,
    pub timestamp: u64,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            mel_bands: [0.0; MEL_BANDS],
            energy: 0.0,
            spectral_flux: 0.0,
            zero_crossing_rate: 0.0,
            is_transient: false,
            timestamp: 0,
        }
    }
}
