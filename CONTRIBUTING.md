# Contributing to Carokia Brain

Thank you for your interest in contributing to Carokia Brain!

## Prerequisites

- **Rust** (stable, 2021 edition) — install via [rustup](https://rustup.rs/)
- **Ollama** — required for LLM features ([ollama.ai](https://ollama.ai/))
- Platform-specific dependencies for optional features (Whisper needs libwhisper, audio needs ALSA/CoreAudio)

## Building

```bash
cargo build --workspace
```

To build with specific features:

```bash
cargo build --workspace --features simulation
```

## Testing

```bash
cargo test --workspace
```

Tests are designed to run without Ollama or any external service.

## Running Demos

```bash
# Chat CLI (needs Ollama running)
cargo run --example chat_cli

# Simulation navigation
cargo run --example sim_nav -p carokia-brain --features simulation

# Guardian mode
cargo run --example guardian_demo -p carokia-brain --features simulation

# Vision demo (needs Ollama + ffmpeg)
cargo run --example vision_demo --features vision
```

## Pull Request Conventions

1. Fork the repository and create a feature branch from `main`.
2. Ensure all checks pass before submitting:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --check
   ```
3. Keep PRs focused — one feature or fix per PR.
4. Add tests for new functionality.
5. Update documentation if you change public APIs.

## Commit Message Format

We follow conventional commits:

```
type(scope): description
```

**Types:** `feat`, `fix`, `docs`, `refactor`, `test`, `ci`, `chore`

**Scopes:** `core`, `brain`, `language`, `memory`, `perception`, `planner`, `decision`, `sim`

**Examples:**
```
feat(planner): add LLM-based goal decomposition
fix(memory): prevent duplicate entries in short-term store
docs(brain): update architecture diagram
ci: add clippy and fmt checks
```

## Code Style

- Run `cargo fmt` before committing.
- All public items should have doc comments.
- Prefer `thiserror` for error types, not ad-hoc strings.
- Use feature flags for heavy optional dependencies (audio, vision, SQLite).

## Architecture

See [docs/architecture.md](docs/architecture.md) for an overview of the crate structure and data flow.
