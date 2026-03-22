# Twitter/X Thread Draft

## Tweet 1 (Hook)
I built an autonomous robot brain entirely in Rust.

It can chat, listen, see, remember, navigate, and guard a space -- all running locally on your machine.

It is open source. Here is what it does and how it works:

github.com/carokia-robotics/carokia-brain

## Tweet 2 (Architecture)
The architecture: 8 Rust crates in a workspace.

- carokia-core: traits and types
- carokia-language: LLM chat (Ollama)
- carokia-perception: vision + audio
- carokia-memory: SQLite persistence
- carokia-planner: goal decomposition
- carokia-decision: behavior engine
- carokia-sim: 2D physics simulation
- carokia-brain: orchestrator

Hexagonal design -- swap any component without touching the rest.

## Tweet 3 (Demos)
7 runnable demos included:

1. Chat CLI -- talk to the robot
2. Voice chat -- speak, it listens via Whisper
3. Vision -- describe scenes from camera or image
4. Memory -- persistent facts across sessions
5. Navigation -- autonomous 2D obstacle avoidance
6. Guardian -- patrol, detect threats, emergency response
7. Demo loop -- all capabilities in sequence

No hardware needed. Everything runs in your terminal.

## Tweet 4 (Why Rust)
Why Rust for robotics?

- Traits let you swap a simulated lidar for a real one: same interface, zero code changes
- No GC pauses during real-time navigation
- Fearless concurrency for parallel perception + planning + actuation
- Compile-time guarantees that Python cannot give you

The type system is the best robotics middleware I have ever used.

## Tweet 5 (Try it)
Try it in 5 minutes:

```
ollama pull gemma3:latest
git clone https://github.com/carokia-robotics/carokia-brain.git
cd carokia-brain
cargo run --example chat_cli
```

Quickstart guide: github.com/carokia-robotics/carokia-brain/blob/main/docs/quickstart.md
Website: carokia-robotics.github.io/carokia-web

MIT licensed. Contributions welcome.

What should I build next?
