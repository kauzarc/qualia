# Qualia

A real-time visual generation engine for live performance (VJing) that uses online learning to map audio to visuals. Instead of hand-coded rules, an AI learns your aesthetic preferences through feedback during the session.

> **Status:** Early development. The audio-to-visual pipeline works end-to-end with a passthrough model. Online learning is not yet implemented.

## How it works

```
Microphone ─→ DSP ─→ Inference ─→ GPU Renderer
                         ↑                 │
                     Trainer ←── Feedback ←┘
```

1. **Audio capture** picks up the live signal and streams raw samples to the DSP thread
2. **DSP** extracts features every ~11ms: 64 mel bands, RMS energy, spectral flux, zero-crossing rate, transient detection (67 floats total)
3. **Inference** maps audio features to normalized visual parameters ([0, 1] control voltages)
4. **Rendering** drives a WGSL shader with those parameters at 60 FPS via wgpu
5. **Feedback** from the performer (like/dislike) feeds into an online training loop that updates the model mid-session

The performer doesn't tweak knobs -- they vote on the output, and the AI converges toward their taste.

## Building and running

```bash
cargo build
cargo run
```

### Requirements

- Rust 1.85+ (edition 2024)
- GPU with Vulkan, Metal, or DX12 support
- Audio input device (ALSA/PipeWire on Linux, CoreAudio on macOS, WASAPI on Windows)

### Debug logging

```bash
RUST_LOG=debug cargo run
```

## Architecture

The system separates threads by temporal domain to keep the audio path lock-free and glitch-free:

| Thread | Rate | Role |
|--------|------|------|
| Audio | 48 kHz (hard real-time) | Zero-allocation sample capture (cpal) |
| DSP | ~90 Hz | Feature extraction (FFT, mel filterbank, temporal features) |
| Inference | >60 Hz | Model forward pass, audio state to visual params |
| Main | 60 FPS | wgpu rendering + egui control panel |
| Trainer | async | Online learning from replay buffer + user feedback |

All high-frequency data flows through lock-free ring buffers (rtrb). The trainer communicates via mpsc and updates the inference model atomically (arc-swap).

## License

TBD
