# Carokia Brain Architecture

## Crate Dependency Graph

```
                         +-----------------+
                         |  carokia-brain  |  (orchestrator)
                         +--------+--------+
                                  |
          +-----------+-----------+-----------+-----------+
          |           |           |           |           |
   +------+---+ +-----+------+ +-+-------+ +-+-------+ +-+--------+
   | language | | perception | | memory  | | planner | | decision |
   +------+---+ +-----+------+ +-+-------+ +-+--+----+ +-+--------+
          |           |           |           |  |        |
          +-----------+-----------+-----------+--+--------+
                                  |
                           +------+------+
                           | carokia-core|  (types, traits, bus, errors)
                           +-------------+

   +-------------+
   | carokia-sim |  (optional, behind `simulation` feature)
   +-------------+
```

All crates depend on `carokia-core` for shared types, error definitions, and trait interfaces.
The `carokia-brain` crate is the top-level orchestrator that wires everything together.

## Data Flow

```
Sensor Input -> Perception -> Memory -> Planner -> Decision -> Action Output
     |              |            |          |           |           |
  SensorFrame   Percepts   MemoryEntry   Tasks    ActionCommand   Action
```

1. **Sensor**: Raw data arrives as `SensorFrame` (vision, audio, lidar, IMU).
2. **Perception**: The `PerceptionPipeline` processes frames into typed `Percept` values (objects, distances, speech).
3. **Memory**: Percepts are stored as `MemoryEntry` in `ShortTermMemory` (in-memory) or `SqliteMemoryStore` (persistent).
4. **Planner**: Goals are decomposed into concrete `Task` steps via `RulePlanner` or `LlmPlanner`.
5. **Decision**: The `BehaviorDecisionEngine` evaluates the current `WorldState` (percepts + goals + memories + tasks) and selects the highest-priority `ActionCommand`.
6. **Action**: The chosen `Action` (Move, Speak, Halt, Actuate) is dispatched to actuators.

## Brain Tick Loop

The `Brain::run()` method drives a continuous cognitive loop at a configurable tick rate (default 10 Hz):

```
loop {
    1. Read sensor frame
    2. Run perception pipeline  -> percepts
    3. Store percepts in memory
    4. Decompose goals into tasks (planner)
    5. Build WorldState from percepts + goals + tasks + memories
    6. Run decision engine       -> ActionCommand
    7. Execute action
    8. Sleep until next tick
}
```

The loop is cancellable via a `CancellationToken` for graceful shutdown.

## Hexagonal Architecture

Carokia follows a hexagonal (ports-and-adapters) pattern:

- **Core domain**: `carokia-core` defines traits (`SensorProcessor`, `MemoryStore`, `LlmBackend`, `Planner`) that are pure interfaces.
- **Adapters**: Concrete implementations live in their respective crates behind feature flags:
  - `OllamaBackend` / `ClaudeBackend` implement `LlmBackend`
  - `SqliteMemoryStore` implements `MemoryStore`
  - `FfmpegCamera` / `FileCamera` implement `CameraSource`
  - `WhisperProcessor` implements speech-to-text
- **Orchestrator**: `carokia-brain` composes adapters into a running system without depending on concrete implementations at the type level.

This means you can swap LLM providers, memory backends, or sensor sources without changing the core logic.

## Feature Flags

| Feature       | Crate              | What it enables                              |
|---------------|--------------------|----------------------------------------------|
| `voice`       | carokia-brain      | Audio capture (cpal) + Whisper STT           |
| `vision`      | carokia-brain      | Image analysis via LLM vision models         |
| `sqlite`      | carokia-brain      | Persistent memory with SQLite                |
| `embeddings`  | carokia-memory     | Vector embeddings via Ollama API             |
| `llm-planner` | carokia-brain      | LLM-based goal decomposition                 |
| `simulation`  | carokia-brain      | 2D world simulation with physics and lidar   |
| `reasoning`   | carokia-brain      | Chain-of-thought reasoning via LLM           |
| `reflection`  | carokia-brain      | Self-reflection and insight generation        |
| `ollama`      | carokia-language   | Ollama LLM backend (default)                 |
| `claude`      | carokia-language   | Anthropic Claude LLM backend                 |
| `audio`       | carokia-perception | Microphone capture and WAV recording         |
| `whisper`     | carokia-perception | Whisper speech-to-text                       |
| `cli`         | carokia-brain      | clap-based CLI argument parsing for examples |

The default build has no heavy optional dependencies, keeping compile times fast.
