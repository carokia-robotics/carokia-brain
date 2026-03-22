# Run Carokia in 5 Minutes

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Ollama](https://ollama.ai/) (for local LLM inference)

## Setup

### 1. Install Ollama and pull a model

```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull gemma3:latest
```

### 2. Clone and build Carokia

```bash
git clone https://github.com/carokia-robotics/carokia-brain.git
cd carokia-brain
cargo build --workspace
```

### 3. Chat with Carokia

```bash
cargo run --example chat_cli
```

Type a message and press Enter. Carokia will respond using the local LLM.
Type `quit` to exit.

## All Demos

### Chat CLI (Sprint 1)
Interactive text conversation with Carokia.
```bash
cargo run --example chat_cli
```

### Voice Chat (Sprint 2)
Speech-to-text conversation using Whisper. Requires a microphone and ffmpeg.
```bash
cargo run --example voice_chat --features voice
```

### Vision Analysis (Sprint 3)
Scene understanding via an LLM vision model. Requires ffmpeg or an image file.
```bash
# Live camera
cargo run --example vision_demo --features vision

# From a file
cargo run --example vision_demo --features vision -- --file photo.jpg --once
```

### Memory and Planning (Sprint 4)
Persistent memory with SQLite and LLM-powered goal decomposition.
```bash
cargo run --example memory_demo --features sqlite,llm-planner
```

### Navigation Simulation (Sprint 5)
Autonomous 2D navigation with lidar-based obstacle avoidance, rendered in ASCII.
```bash
cargo run --example sim_nav -p carokia-brain --features simulation
```

### Guardian Mode (Sprint 6)
Full autonomous patrol with threat detection and emergency response.
```bash
cargo run --example guardian_demo -p carokia-brain --features simulation
```

### Demo Loop
Cycles through multiple capabilities in sequence.
```bash
cargo run --example demo_loop -p carokia-brain
```

## Docker (Quick Alternative)

If you have Docker installed, you can skip the Rust toolchain entirely:

```bash
docker compose up -d ollama
docker compose run --rm carokia
```

## Troubleshooting

### Ollama connection refused
Make sure Ollama is running. Start it with:
```bash
ollama serve
```
By default it listens on `http://localhost:11434`. If you changed the host, set `OLLAMA_HOST` in your environment or update `carokia.toml`.

### Whisper / voice features fail to compile
The `whisper-rs` crate requires a C compiler and CMake:
```bash
# macOS
xcode-select --install
brew install cmake

# Ubuntu/Debian
sudo apt-get install build-essential cmake
```

### Vision demo shows "no camera found"
The vision demo uses ffmpeg to capture frames. Install it:
```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt-get install ffmpeg
```
Alternatively, pass an image file with `--file photo.jpg --once`.

### Cargo build fails with linker errors on Linux
Install the standard development libraries:
```bash
sudo apt-get install pkg-config libssl-dev libasound2-dev
```

### Model not found
Ensure you pulled the model Ollama expects. Check `carokia.toml` for the configured model name, or pull the default:
```bash
ollama pull gemma3:latest
```
