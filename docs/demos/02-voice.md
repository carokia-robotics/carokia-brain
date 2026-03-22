# Demo Script: Voice Conversation

## Overview
Show Carokia listening to speech through a microphone, transcribing it with Whisper, and responding via the LLM. This demonstrates the full voice interaction loop.

## Prerequisites
- Ollama running with a chat model
- A working microphone
- ffmpeg installed
- Whisper model downloaded (happens automatically on first run)

## Commands

```bash
# Build and run with voice features enabled
cargo run --example voice_chat --features voice
```

## Walkthrough

1. **Launch** -- the demo prints a prompt indicating it is listening.

2. **Speak a greeting** -- say: "Hello Carokia, how are you?"
   - Expected: Whisper transcribes the audio, the transcript is printed, and Carokia responds via the LLM.

3. **Ask a question** -- say: "What is the weather like outside?"
   - Expected: Carokia acknowledges it cannot check real sensors yet but describes what it would do with weather sensors.

4. **Test noise handling** -- clap or make a short noise.
   - Expected: Whisper may transcribe silence or noise; the system handles it gracefully without crashing.

5. **Exit** -- press Ctrl+C to stop.

## Talking Points

- Whisper runs entirely on-device using `whisper-rs` (C++ bindings via whisper.cpp). No cloud APIs.
- Audio capture uses `cpal` for cross-platform microphone access.
- The voice pipeline is modular: swap the STT engine by implementing the `SensorSource` trait for audio frames.
- Latency depends on model size -- the base Whisper model gives near-real-time results on Apple Silicon.
