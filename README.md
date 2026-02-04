# Qualia

Real-time visual generation engine driven by online learning, designed for VJing and live performance.

## Concept

Qualia creates an AI layer between audio input and visual output. Instead of explicit rules ("if bass > threshold, flash red"), the system learns from user feedback during the session. The performer votes (like/dislike) on the visuals, and the AI adapts to their aesthetic preferences in real-time.

## Architecture

Asynchronous multi-threaded architecture separating temporal domains to guarantee low latency on the critical audio/visual path while allowing background training.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Audio Thread (Hard Real-time)                                          │
│  ┌─────────────┐                                                        │
│  │ AudioDriver │  cpal callback, zero-allocation                        │
│  └──────┬──────┘                                                        │
│         │ raw samples (rtrb)                                            │
└─────────┼───────────────────────────────────────────────────────────────┘
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  DSP Thread (~90 Hz)                                                    │
│  ┌─────────────┐                                                        │
│  │  DspEngine  │  FFT, Mel spectrogram, energy, transient detection     │
│  └──────┬──────┘                                                        │
│         │ AudioState [67 floats] (rtrb)                                 │
└─────────┼───────────────────────────────────────────────────────────────┘
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Inference Thread (>60 Hz)                                              │
│  ┌─────────────┐                                                        │
│  │  Inference  │  ONNX model forward pass (ort)                         │
│  └──────┬──────┘                                                        │
│         │ VisualParams [N floats] (rtrb)                                │
└─────────┼───────────────────────────────────────────────────────────────┘
          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  Main Thread (60 FPS)                                                   │
│  ┌─────────────┐    ┌─────────────┐                                     │
│  │   Display   │    │    egui     │  feedback controls                  │
│  │   (wgpu)    │    │  (control)  │──────────────────┐                  │
│  └─────────────┘    └─────────────┘                  │                  │
└──────────────────────────────────────────────────────┼──────────────────┘
                                                       │ Feedback (mpsc)
┌──────────────────────────────────────────────────────┼──────────────────┐
│  Trainer Thread (async, low priority)                ▼                  │
│  ┌─────────────┐    ┌──────────────┐                                    │
│  │   Trainer   │◄───│ ReplayBuffer │  history + delayed reward assign   │
│  │    (ort)    │    │              │                                    │
│  └──────┬──────┘    └──────────────┘                                    │
│         │                                                               │
│         │ export updated model                                          │
│         ▼                                                               │
│  Inference reloads via arc-swap                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Module Status

| Module | Thread | Communication | Status |
|--------|--------|---------------|--------|
| **AudioDriver** | Audio | `rtrb` → DSP | Basic infrastructure |
| **DspEngine** | DSP | `rtrb` → Inference | Basic infrastructure |
| **Inference** | Inference | `rtrb` → Main | Basic infrastructure |
| **Display** | Main | renders VisualParams, sends Feedback | Basic infrastructure |
| **Trainer** | Trainer | `mpsc` ← Main | Basic infrastructure |

## Building

```bash
cargo build
```

## Running

```bash
cargo run
```

## Dependencies

- Rust 1.85+ (edition 2024)
- GPU with Vulkan/Metal/DX12 support
