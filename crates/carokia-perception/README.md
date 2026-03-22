# carokia-perception

Sensor processing pipeline for Carokia. Converts raw `SensorFrame` data into structured `Percept` values. Supports vision analysis and face detection via LLM vision models (behind the `vision` feature), audio capture and WAV recording (behind the `audio` feature), and speech-to-text via Whisper (behind the `whisper` feature). A stub processor is included for testing without hardware.
