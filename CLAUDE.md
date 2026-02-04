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
    │ raw samples [f64; 512] via rtrb
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

- `session.rs` - Orchestrates all threads
- `channel/` - Pipeline communication infrastructure
  - `pipe.rs` - `Processor` trait and `Pipe<P>` for "drain to latest" buffer strategy
  - `channels.rs` - `Channels` struct creates all ring buffers
- `audio/` - Hard real-time audio capture (cpal)
  - `driver.rs` - `AudioDriver` with `HopAccumulator`
- `dsp/` - Digital signal processing
  - `engine.rs` - `DspEngine` thread management
  - `processor.rs` - `DspProcessor` implements `Processor`
  - `state.rs` - `AudioState` (67 floats)
- `inference/` - Neural network inference
  - `engine.rs` - `Inference` thread management
  - `model.rs` - `InferenceModel` trait (infer, output_size, reward)
  - `processor.rs` - `InferenceProcessor` wraps model for `Pipe`
  - `passthrough.rs` - `PassthroughModel` (MVP placeholder)
  - `params.rs` - `VisualParams`, `ControlVoltage`
- `trainer.rs` - Online learning with replay buffer (skeleton)
- `display/` - GPU rendering (wgpu) and control panel (egui)
  - `gpu.rs` - `GpuContext` for device/queue management
  - `window.rs` - `WindowContext` for surface configuration
  - `view.rs` - `ViewWindow` for visual output
  - `control/` - Control panel window
    - `panel.rs` - `ControlPanel` UI with feedback button
    - `gui.rs` - `GuiContext` egui state and rendering
    - `frame.rs` - `PreparedFrame` RAII texture management

### Key Constants

```rust
// channel/channels.rs
const DSP_RATE_HZ: usize = 90;
const INFERENCE_RATE_HZ: usize = 60;

// audio.rs
pub const HOP_SIZE: usize = 512;  // ~10.7ms at 48kHz

// dsp.rs
pub const MEL_BANDS: usize = 64;

// inference/params.rs
pub const MAX_ACTIONS: usize = 64;
```

### Type-Safe Domain Values

- `ControlVoltage(f64)` - Normalized [0.0, 1.0] for shader parameters
- `Reward(f64)` - User feedback [-1.0, 1.0]
- `Feedback` - Contains `Reward` and timestamp, sent from Display to Trainer

### Communication Patterns

- **rtrb ring buffers** - Lock-free for high-frequency data (samples, audio state, visual params)
- **std::mpsc** - For low-frequency feedback events (Display → Trainer)
- **EventLoopProxy** - UI events to main thread (ControlPanel → Session via `AppEvent`)
- **arc-swap** - Atomic model reloading

### Key Abstractions

- **`Processor` trait** - Generic transform with `Input`/`Output` associated types
- **`Pipe<P: Processor>`** - Connects ring buffers with "drain to latest" strategy
- **`InferenceModel` trait** - Domain-specific: `infer()`, `output_size()`, `reward()`

### Design Principles

- Zero-allocation in audio callback (fixed-size buffers)
- Message passing over locks for thread safety
- Composition-based rendering with wgpu command encoders
- Error handling: `thiserror` for custom errors, `anyhow` for Results
- Logging: `tracing` macros (info!, debug!, error!)
