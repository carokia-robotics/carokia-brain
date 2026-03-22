# r/robotics Post

## Title
Carokia: open-source autonomous robot brain with vision, speech, navigation, and guardian mode

## Body

I have been working on Carokia, an open-source AI brain for autonomous robots. It is written in Rust and designed to eventually run on physical hardware (quadruped, aerial, aquatic platforms), but right now it works as a fully functional software stack with a built-in 2D simulation.

**What it can do today:**
- **Chat** with a local LLM (Ollama) -- the robot has a personality and conversation memory
- **Listen** via Whisper speech-to-text running on-device
- **See** using LLM vision models for scene understanding and face detection
- **Remember** facts and experiences in a persistent SQLite database
- **Navigate** autonomously in a 2D simulated world with lidar-based obstacle avoidance
- **Guard** a space with waypoint patrol, threat detection, and emergency escalation

**Why I built it:**
Most robotics AI stacks are Python-based, which is fine for prototyping but painful at deployment. I wanted to see if a statically typed, zero-cost-abstraction language could handle the full stack -- perception, reasoning, planning, actuation -- without sacrificing developer ergonomics. So far, Rust has been a strong fit.

**Architecture:**
8 crates in a workspace: core traits, language/LLM, perception (vision + audio), memory, planning, decision/behavior engine, 2D simulation, and the top-level brain orchestrator. Everything communicates through defined trait interfaces, so swapping a simulated sensor for a real one is a one-line change.

**Try it (5 minutes):**
```bash
ollama pull gemma3:latest
git clone https://github.com/carokia-robotics/carokia-brain.git
cd carokia-brain
cargo run --example guardian_demo -p carokia-brain --features simulation
```

**Links:**
- Repository: https://github.com/carokia-robotics/carokia-brain
- Quickstart: https://github.com/carokia-robotics/carokia-brain/blob/main/docs/quickstart.md
- Website: https://carokia-robotics.github.io/carokia-web

Looking for feedback, collaborators, and ideas. Particularly interested in hearing from anyone working on ROS2 integration or embedded Rust for robotics hardware. MIT licensed.
