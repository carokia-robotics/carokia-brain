# Demo Script: Vision Analysis

## Overview
Demonstrate Carokia analyzing images -- either from a live camera feed or a static file. The LLM vision model describes scenes, detects objects, and identifies faces.

## Prerequisites
- Ollama running with a vision-capable model (e.g., `llava` or `gemma3`)
- ffmpeg (for live camera) or an image file

## Commands

```bash
# Live camera mode
cargo run --example vision_demo --features vision

# Single image analysis
cargo run --example vision_demo --features vision -- --file photo.jpg --once
```

## Walkthrough

1. **Single image mode** -- run with `--file test.jpg --once`.
   - Expected: Carokia prints a detailed scene description (objects, people, setting, colors).

2. **Live camera mode** -- run without `--file`.
   - Expected: Carokia captures frames periodically and prints scene descriptions in a loop.

3. **Face detection** -- point camera at a person or use a photo with faces.
   - Expected: The face detector identifies face bounding boxes; the LLM describes the people in the scene.

4. **Exit** -- press Ctrl+C (live mode) or the demo exits after one frame (with `--once`).

## Talking Points

- Vision uses Ollama's multimodal capabilities -- images are base64-encoded and sent alongside a text prompt.
- The face detector is a separate module that can run independently of the LLM for low-latency detection.
- Camera abstraction supports multiple backends: ffmpeg capture, file input, or future hardware cameras.
- All processing is local -- images never leave the device.
