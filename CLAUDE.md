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
    │ raw f64 samples via rtrb
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
  - `channels.rs` - `Channels` struct creates all ring buffers
- `audio/` - Hard real-time audio capture (cpal)
  - `driver.rs` - `AudioDriver` bulk-writes raw f64 samples
- `dsp/` - Digital signal processing
  - `thread.rs` - `DspThread` thread spawn and join
  - `orchestrator.rs` - `DspOrchestrator` core loop, coordinates IO and processing
  - `accumulator.rs` - `HopAccumulator` double-buffered sample-to-hop accumulation
  - `input.rs` - `AudioInput` drains samples hop-by-hop via `drain_to_next_hop()`
  - `processor.rs` - `DspProcessor` stateful feature extraction
  - `state.rs` - `AudioState` (67 floats)
- `inference/` - Neural network inference
  - `engine.rs` - `Inference` thread management
  - `model.rs` - `InferenceModel` trait (infer, output_size, reward)
  - `pipe.rs` - `InferencePipe` with `TickResult` for IO
  - `passthrough.rs` - `PassthroughModel` (MVP placeholder)
  - `params.rs` - `VisualParams`, `ControlVoltage`
- `trainer.rs` - Online learning with replay buffer (skeleton)
- `display/` - GPU rendering (wgpu) and control panel (egui)
  - `gpu.rs` - `GpuContext` for device/queue management
  - `window.rs` - `WindowContext` for surface configuration
  - `view.rs` - `ViewWindow` for visual output
  - `params.rs` - `ParamsBuffer` for visual params with interpolation
  - `ring_pair.rs` - `RingPair<T>` fixed-size circular buffer of 2
  - `control/` - Control panel window
    - `panel.rs` - `ControlPanel` UI with feedback button
    - `gui.rs` - `GuiContext` egui state and rendering
    - `frame.rs` - `PreparedFrame` RAII texture management

### Key Constants

```rust
// channel/channels.rs
const DSP_RATE_HZ: usize = 90;
const INFERENCE_RATE_HZ: usize = 60;

// dsp.rs
pub const HOP_SIZE: usize = 512;  // ~10.7ms at 48kHz
pub const MEL_BANDS: usize = 64;

// inference/params.rs
pub const MAX_ACTIONS: usize = 64;
```

### Type-Safe Domain Values

- `ControlVoltage(f64)` - Normalized [0.0, 1.0] for shader parameters
- `Reward(f64)` - User feedback [-1.0, 1.0]
- `Feedback` - Contains `Reward` and `Instant` timestamp, sent from Display to Trainer
- Timestamps use `std::time::Instant` for monotonic timing across threads

### Communication Patterns

- **rtrb ring buffers** - Lock-free for high-frequency data (samples, audio state, visual params)
- **std::mpsc** - For low-frequency feedback events (Display → Trainer)
- **EventLoopProxy** - UI events to main thread (ControlPanel → Session via `AppEvent`)
- **arc-swap** - Atomic model reloading

### Key Abstractions

- **`HopAccumulator`** - Double-buffered accumulator, zero-copy hop completion via index toggle
- **`AudioInput`** - Drains samples hop-by-hop, returns `Option<&AudioSamples>` per hop
- **`DspOrchestrator`** - Processes every hop sequentially, sleeps when idle
- **`InferencePipe`** - Module-local IO with "drain to latest" strategy
- **`TickResult`** - Enum for inference pipe tick outcomes (Produced, NoInput, BufferFull)
- **`RingPair<T>`** - Fixed-size circular buffer of 2 with O(1) push
- **`InferenceModel` trait** - Domain-specific: `infer()`, `output_size()`, `reward()`

### Design Principles

- Minimal work in audio callback (bulk-write raw samples, no framing)
- Message passing over locks for thread safety
- Composition-based rendering with wgpu command encoders
- Error handling: `thiserror` for custom errors, `anyhow` for Results
- Logging: `tracing` macros (info!, debug!, error!)
