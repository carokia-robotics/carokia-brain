# r/rust Post

## Title
Show HN-style: Carokia -- an autonomous robot brain written entirely in Rust

## Body

I have been building Carokia, an open-source autonomous robot brain in Rust. It started as an experiment in whether Rust's type system and async ecosystem could handle the complexity of a real robotics stack, and it has grown into something I am genuinely excited to share.

**What it does:**
Carokia integrates LLM-powered conversation (via Ollama), speech recognition (Whisper), computer vision, persistent memory (SQLite), autonomous navigation, and a full guardian/security mode -- all in a single workspace of 8 crates.

**Why Rust:**
- Trait-based architecture makes swapping components trivial. The `SensorSource` trait works for a simulated lidar, a webcam, or a microphone -- same interface, different backends.
- Fearless concurrency matters when you have perception, planning, and actuation running simultaneously.
- No garbage collector pauses during real-time navigation and threat detection.
- The type system catches integration bugs at compile time that would be runtime crashes in Python.

**Architecture:**
The workspace is organized as a hexagonal (ports-and-adapters) architecture: `carokia-core` defines the traits, each domain crate implements them, and `carokia-brain` orchestrates everything. The simulation crate lets you test navigation and guardian behaviors without any hardware.

**Try it:**
```bash
ollama pull gemma3:latest
git clone https://github.com/carokia-robotics/carokia-brain.git
cd carokia-brain
cargo run --example chat_cli
```

There are 7 runnable demos covering chat, voice, vision, memory, navigation simulation, and autonomous guardian patrol.

**Links:**
- Repository: https://github.com/carokia-robotics/carokia-brain
- Quickstart: https://github.com/carokia-robotics/carokia-brain/blob/main/docs/quickstart.md
- Website: https://carokia-robotics.github.io/carokia-web

I would love feedback on the architecture, the trait design, or ideas for what to build next. The project is MIT licensed and contributions are welcome.
