# carokia-memory

Memory subsystem for the Carokia robot brain. Includes an in-memory short-term store with configurable capacity, an optional SQLite-backed persistent store (behind the `sqlite` feature), and vector embedding support via the Ollama API (behind the `embeddings` feature). Memory entries carry a kind, content string, importance score, and timestamp for recall and reflection.
