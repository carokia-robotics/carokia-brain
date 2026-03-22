# Hacker News Post

## Title
Show HN: Carokia -- Autonomous robot brain in Rust (LLM, vision, navigation, guardian mode)

## Body

Carokia is an open-source AI brain for autonomous robots, written entirely in Rust. It integrates local LLM inference (Ollama), speech recognition (Whisper), computer vision, persistent memory, autonomous navigation, and a full guardian/security mode.

The project is structured as a Cargo workspace with 8 crates following a hexagonal architecture. Core traits define the interfaces -- `SensorSource`, `Actuator`, `MessageBus` -- and each domain crate provides concrete implementations. This means you can swap a simulated lidar for a real one without touching the navigation logic.

Some things I found interesting while building this:

1. Rust's async ecosystem handles concurrent perception + planning + actuation well. Tokio tasks for each subsystem, channels for communication, no data races by construction.

2. The trait-based design paid off when adding the simulation crate. The navigation code that works in simulation will work on hardware with zero changes -- just a different `SensorSource` implementation.

3. Local LLM inference via Ollama means the robot's "brain" runs entirely on-device. No cloud dependency, no API keys, no latency penalty for simple tasks.

4. The guardian mode (autonomous patrol + threat detection + emergency response) runs without any LLM at all -- pure Rust deterministic logic. The LLM is only used for conversation and open-ended reasoning.

**Quick start:**
```
ollama pull gemma3:latest
git clone https://github.com/carokia-robotics/carokia-brain.git
cd carokia-brain
cargo run --example chat_cli
```

7 demos included. No hardware required -- everything runs in a terminal.

- Repo: https://github.com/carokia-robotics/carokia-brain
- Quickstart: https://github.com/carokia-robotics/carokia-brain/blob/main/docs/quickstart.md
- Website: https://carokia-robotics.github.io/carokia-web

MIT licensed. Feedback welcome.
