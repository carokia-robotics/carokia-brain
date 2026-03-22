//! Text-to-speech support.
//!
//! Provides a `TextToSpeech` trait and a `SystemTts` implementation
//! that uses the macOS `say` command.

use async_trait::async_trait;
use carokia_core::BrainError;

/// Trait for text-to-speech backends.
#[async_trait]
pub trait TextToSpeech: Send + Sync {
    /// Speak the given text aloud. Blocks until speech is complete.
    async fn speak(&self, text: &str) -> Result<(), BrainError>;
}

/// TTS backend using the macOS `say` command.
///
/// This is a simple, zero-dependency TTS that works on any Mac.
/// For other platforms, implement the `TextToSpeech` trait with
/// an appropriate backend (e.g. espeak, piper, etc.).
pub struct SystemTts;

impl SystemTts {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemTts {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TextToSpeech for SystemTts {
    async fn speak(&self, text: &str) -> Result<(), BrainError> {
        let status = tokio::process::Command::new("say")
            .arg(text)
            .status()
            .await
            .map_err(|e| BrainError::Internal(format!("TTS error: {e}")))?;

        if !status.success() {
            return Err(BrainError::Internal("TTS command failed".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_tts_can_be_constructed() {
        let _tts = SystemTts::new();
    }

    #[test]
    fn system_tts_default_works() {
        let _tts = SystemTts::default();
    }

    /// A mock TTS for testing that the trait works.
    struct MockTts {
        spoken: std::sync::Mutex<Vec<String>>,
    }

    impl MockTts {
        fn new() -> Self {
            Self {
                spoken: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl TextToSpeech for MockTts {
        async fn speak(&self, text: &str) -> Result<(), BrainError> {
            self.spoken.lock().unwrap().push(text.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn mock_tts_records_speech() {
        let tts = MockTts::new();
        tts.speak("Hello").await.unwrap();
        tts.speak("World").await.unwrap();
        let spoken = tts.spoken.lock().unwrap();
        assert_eq!(spoken.len(), 2);
        assert_eq!(spoken[0], "Hello");
        assert_eq!(spoken[1], "World");
    }
}
