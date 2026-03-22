# Carokia Brain

**AI core and decision engine** — a Rust-based inference, planning, and robotics runtime.

Carokia is an autonomous robot brain built from the ground up in Rust. It integrates
language understanding, perception (vision, audio, lidar), memory, planning, decision-making,
simulation, and guardian-mode security into a single coherent architecture.

## Architecture

```
                        +-------------------+
                        |   carokia-brain   |  Orchestrator / main loop
                        +--------+----------+
                                 |
          +----------+-----------+-----------+----------+
          |          |           |           |          |
   +------+--+ +----+-----+ +---+----+ +----+---+ +---+------+
   |  core   | | language | | memory | | planner| | decision |
   | types,  | |  LLM,    | | short- | |  rule  | | behavior |
   | traits, | |  chat,   | | term,  | |  & LLM | | engine,  |
   | bus,    | |  TTS     | | SQLite | |  goals | | patrol,  |
   | errors  | |          | |        | |        | | threat,  |
   +---------+ +----------+ +--------+ +--------+ | emergency|
                                                   +----+-----+
                                                        |
                                              +---------+--------+
                                              |   carokia-sim    |
                                              | 2D world, robot, |
                                              | physics, lidar,  |
                                              | ASCII renderer   |
                                              +------------------+
   +-------------------+
   | carokia-perception |
   | vision, audio,    |
   | whisper, camera,  |
   | face detection    |
   +-------------------+
```

## Sprints

| Sprint | Name           | Capabilities                                       |
|--------|----------------|-----------------------------------------------------|
| 1      | The Voice      | LLM chat via Ollama (chat_cli)                      |
| 2      | The Listener   | Speech-to-text with Whisper, voice conversations    |
| 3      | The Eye        | Vision processing, scene understanding              |
| 4      | The Mind       | Persistent memory (SQLite), LLM-powered planning    |
| 5      | The Navigator  | 2D simulation, lidar, autonomous obstacle avoidance |
| 6      | The Guardian   | Patrol, threat detection, alerts, emergency response|

## Prerequisites

- **Rust** (stable, 2021 edition) — install via [rustup](https://rustup.rs/)
- **Ollama** — required for LLM features ([ollama.ai](https://ollama.ai/))
- Platform-specific dependencies for optional features (Whisper, audio, etc.)

## Quick Start

```bash
# Clone and build
git clone <repo-url> carokia-brain
cd carokia-brain
cargo build --workspace

# Run tests
cargo test --workspace
```

## Demos

### Sprint 1 — Chat CLI
Interactive conversation with an LLM backend.
```bash
cargo run --example chat_cli
```

### Sprint 2 — Voice Chat
Speech-to-text conversation using Whisper.
```bash
cargo run --example voice_chat --features voice
```

### Sprint 3 — Vision Demo
Scene understanding and object detection via vision models.
```bash
cargo run --example vision_demo --features vision
```

### Sprint 4 — Memory Demo
Persistent memory and LLM-powered planning.
```bash
cargo run --example memory_demo --features sqlite,llm-planner
```

### Sprint 5 — Simulation Navigation
Autonomous navigation in a 2D simulated world with lidar-based obstacle avoidance.
```bash
cargo run --example sim_nav -p carokia-brain --features simulation
```

### Sprint 6 — Guardian Mode
Full autonomous guardian: waypoint patrol, threat detection, emergency response,
and a real-time ASCII terminal dashboard.
```bash
cargo run --example guardian_demo -p carokia-brain --features simulation
```

The guardian demo shows a robot patrolling waypoints around a room. After 50 ticks
an intruder is spawned. The threat detection system escalates through Suspicious to
Confirmed, triggering an emergency halt and alert — all rendered live in the terminal.

## Crate Overview

| Crate                | Purpose                                            |
|----------------------|----------------------------------------------------|
| `carokia-core`       | Shared types, traits, event bus, error definitions  |
| `carokia-language`   | LLM backends (Ollama, Claude), conversation, TTS   |
| `carokia-memory`     | Short-term memory, SQLite persistence, embeddings   |
| `carokia-perception` | Sensor processing: vision, audio, lidar, Whisper    |
| `carokia-planner`    | Goal decomposition, rule-based and LLM planning     |
| `carokia-decision`   | Behavior engine, patrol, threat detection, emergency|
| `carokia-sim`        | 2D physics simulation, robot, world, ASCII renderer |
| `carokia-brain`      | Top-level orchestrator, alert manager, demo examples|

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Ensure `cargo test --workspace` and `cargo clippy --workspace` pass
4. Submit a pull request

## License

MIT
