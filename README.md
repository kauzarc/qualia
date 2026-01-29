# Qualia

Real-time visual generation engine driven by online learning, designed for VJing and live performance.

## Concept

Qualia creates an AI layer between audio input and visual output. Instead of explicit rules ("if bass > threshold, flash red"), the system learns from user feedback during the session. The performer votes (like/dislike) on the visuals, and the AI adapts to their aesthetic preferences in real-time.

## Architecture

| Module | Responsibility | Status |
|--------|----------------|--------|
| **Audio** | Capture audio via OS callback | Skeleton |
| **DSP** | FFT, Mel spectrogram, feature extraction | Not started |
| **Inference** | Neural network (audio → visual params) | Not started |
| **Display** | Window management, GPU rendering | Basic infrastructure |
| **Trainer** | Background learning from user feedback | Not started |

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Dependencies

- Rust 1.76+
- GPU with Vulkan/Metal/DX12 support
