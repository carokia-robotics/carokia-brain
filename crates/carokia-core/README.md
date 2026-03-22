# carokia-core

Shared foundation for the Carokia robotics runtime. This crate defines the core types (`SensorFrame`, `Action`, `Timestamp`, `Modality`), traits (`SensorProcessor`, `MemoryStore`, `LlmBackend`), the event bus, and the unified `BrainError` type. All other Carokia crates depend on `carokia-core` for their shared interfaces.
