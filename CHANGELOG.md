# Changelog

All notable changes to the Carokia Brain project are documented here.

## [v0.1.0] — 2026-03-22

First public release of Carokia — an open-source, multi-domain autonomous robot companion built entirely in Rust.

### Sprint 1 — The Voice
- Added `OllamaBackend` adapter connecting to local Ollama instance
- Added `ClaudeBackend` adapter calling Anthropic Messages API via reqwest
- TOML-based `LlmProviderConfig` with provider selection (Ollama/Claude/Mock)
- System prompt support in `ConversationManager`
- Factory function `create_backend()` for dynamic backend selection
- Interactive CLI chat binary (`cargo run --example chat_cli`)

### Sprint 2 — The Ears & Mouth
- `AudioBuffer` type in carokia-core (samples, sample_rate, channels)
- `MicrophoneSource` for audio capture via cpal (feature-gated)
- `WhisperTranscriber` for speech-to-text via whisper-rs with 16kHz resampling
- `TextToSpeech` trait with `SystemTts` implementation (macOS `say` command)
- Voice conversation demo (`cargo run --example voice_chat --features voice`)

### Sprint 3 — The Eyes
- `CameraSource` trait with `FfmpegCamera`, `FileCamera`, `MemoryCamera` implementations
- `VisionAnalyzer` using Ollama multimodal models (gemma3/llava) for scene description
- `FaceDetector` using LLM vision for face detection and description
- `PerceptContent::SceneDescription` and `PerceptContent::FaceDetection` variants
- Vision demo (`cargo run --example vision_demo --features vision`)

### Sprint 4 — The Mind
- `SqliteMemory` implementing `MemoryStore` with bundled SQLite persistence
- `OllamaEmbedder` for vector embeddings via Ollama `/api/embeddings`
- Cosine similarity search for semantic memory recall
- `LlmPlanner` for LLM-powered goal decomposition with robust JSON extraction
- Memory and planning demo (`cargo run --example memory_demo --features sqlite,llm-planner`)

### Sprint 5 — The Simulation
- New `carokia-sim` crate: 2D world with walls, objects, physics
- `SimRobot` with position, heading, 12-ray lidar, collision detection
- Ray-casting engine for lidar simulation and wall intersection
- ASCII terminal renderer (walls `#`, robot `@`, target `T`, obstacles `O`)
- Autonomous navigation with target seeking + obstacle avoidance
- Navigation demo (`cargo run --example sim_nav --features simulation`)

### Sprint 6 — The Guardian
- `PatrolBehavior` with waypoint-based autonomous patrol
- `ThreatDetectionBehavior` with sustained detection confirmation
- `EmergencyResponseBehavior` for halt + alarm on confirmed threat
- `ThreatLevel` enum (None, Suspicious, Confirmed) in WorldState
- `AlertManager` for tracking and logging security alerts
- Guardian demo with terminal dashboard (`cargo run --example guardian_demo --features simulation`)

### Sprint 7 — The Conversationalist
- `StreamingLlmBackend` trait with Ollama and Claude SSE implementations
- Tool use framework: `Tool` trait, `ToolRegistry`, 4 built-in tools (CurrentTime, Calculator, ShellCommand, MemorySearch)
- `PersonalityConfig` with configurable name, traits, speaking style, backstory
- Memory-augmented conversation context (`set_context()` / `clear_context()`)
- `build_personality_prompt()` for dynamic system prompt generation
- Conversationalist demo (`cargo run --example conversationalist_demo --features streaming`)

### Sprint 8 — The Agent
- `ChainOfThoughtReasoner` with ReAct-style loop (Thought → Action → Observation → Answer)
- `EventDrivenBehavior` with configurable triggers (PersonDetected, LoudNoise, TimerElapsed)
- `EmotionalState` model (valence/arousal/dominance) with decay, mood labels, prompt modifiers
- `ReflectionEngine` for periodic memory consolidation via LLM
- Agent demo (`cargo run --example agent_demo --features reasoning`)

### Sprint 9 — The Ship
- GitHub Actions CI pipeline (test, clippy, fmt)
- All 8 crates prepared for crates.io (description, keywords, categories, README)
- Architecture documentation with ASCII diagrams (`docs/architecture.md`)
- CONTRIBUTING.md with development workflow
- Issue and PR templates (`.github/ISSUE_TEMPLATE/`, `.github/PULL_REQUEST_TEMPLATE.md`)
- CLI argument parsing with clap for all examples (feature-gated)
- Error handling improvements (actionable messages, `From<std::io::Error>`)

### Sprint 10 — The Beacon
- Quickstart guide (`docs/quickstart.md`) — run Carokia in 5 minutes
- 6 video demo scripts (`docs/demos/`)
- Dockerfile + docker-compose.yml (multi-stage build with Ollama)
- Release workflow for macOS + Linux binaries (`.github/workflows/release.yml`)
- Social media launch drafts (`docs/launch/`) — Reddit, HN, Twitter
- Blog post on carokia-web: "Building an AI Robot Companion in Rust"
- mdsvex blog support added to carokia-web

### Project Stats
- **155 tests**, 0 failures, 0 clippy warnings
- **9 crates** in Rust workspace
- **10 demos/examples**
- **64 tracked issues** on GitHub project board
- **33 repos** in carokia-robotics GitHub org
- **25 reference projects** analyzed in learning hub

[v0.1.0]: https://github.com/carokia-robotics/carokia-brain/releases/tag/v0.1.0
