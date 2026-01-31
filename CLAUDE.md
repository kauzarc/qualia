# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build                    # Debug build
cargo run                      # Run application
RUST_LOG=debug cargo run       # Run with debug logging
cargo test                     # Run tests
cargo fmt                      # Format code
cargo clippy                   # Lint (pedantic warnings enforced)
cargo build --release          # Release build
```

## Architecture Overview

Qualia is a real-time visual generation engine for VJing that uses online learning to map audio input to visual output. The system separates threads by temporal domain with lock-free communication.

### Thread Architecture

```
Audio Thread (hard real-time, cpal)
    │ raw samples [f32; 512] via rtrb
    ▼
DSP Thread (~90 Hz)
    │ AudioState [67 floats] via rtrb
    ▼
Inference Thread (>60 Hz)
    │ VisualParams via rtrb
    ▼
Main Thread (60 FPS, wgpu/egui)
    │ Feedback via mpsc
    ▼
Trainer Thread (async, low priority)
    │ updated model via arc-swap
    └──► Inference reloads
```

### Module Structure

- `session.rs` - Orchestrates all threads and channels
- `audio.rs` - AudioDriver for hard real-time audio capture (cpal)
- `dsp.rs` - DspEngine for FFT, Mel spectrogram, feature extraction
- `inference.rs` - ONNX model forward pass (ort)
- `trainer.rs` - Online learning with replay buffer
- `display/` - GPU rendering (wgpu) and control panel (egui)
  - `context/` - GPU, window, and GUI context management
  - `render/` - ViewRenderer (visuals) and ControlRenderer (egui)

### Key Constants

```rust
// session.rs
const DSP_RATE_HZ: usize = 90;
const INFERENCE_RATE_HZ: usize = 60;

// audio.rs
pub const HOP_SIZE: usize = 512;  // ~10.7ms at 48kHz

// dsp.rs
pub const MEL_BANDS: usize = 64;

// inference.rs
pub const MAX_ACTIONS: usize = 64;
```

### Type-Safe Domain Values

- `ControlVoltage(f32)` - Normalized [0.0, 1.0] for shader parameters
- `Reward(f32)` - User feedback [-1.0, 1.0]

### Communication Patterns

- **rtrb ring buffers** - Lock-free for high-frequency data (samples, audio state, visual params)
- **std::mpsc** - For low-frequency feedback events
- **arc-swap** - Atomic model reloading

### Design Principles

- Zero-allocation in audio callback (fixed-size buffers)
- Message passing over locks for thread safety
- Composition-based rendering with wgpu command encoders
- Error handling: `thiserror` for custom errors, `anyhow` for Results
- Logging: `tracing` macros (info!, debug!, error!)
