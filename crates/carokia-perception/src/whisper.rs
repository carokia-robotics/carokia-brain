//! Speech-to-text via whisper-rs.
//!
//! Gated behind the `whisper` feature flag.
//!
//! # Model download
//! ```sh
//! mkdir -p models
//! curl -L -o models/ggml-base.bin \
//!   https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
//! ```

use carokia_core::{AudioBuffer, BrainError};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Transcribes audio buffers to text using a Whisper model.
pub struct WhisperTranscriber {
    ctx: WhisperContext,
}

impl WhisperTranscriber {
    /// Create a new transcriber from a ggml model file path.
    pub fn new(model_path: &str) -> Result<Self, BrainError> {
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| BrainError::Perception(format!("Failed to load whisper model: {e}")))?;

        tracing::info!(model_path, "Whisper model loaded");
        Ok(Self { ctx })
    }

    /// Transcribe an AudioBuffer to text.
    ///
    /// The audio is resampled to 16 kHz mono as required by Whisper.
    pub fn transcribe(&self, audio: &AudioBuffer) -> Result<String, BrainError> {
        let mono_16k = resample_to_16k_mono(audio);

        let mut state = self
            .ctx
            .create_state()
            .map_err(|e| BrainError::Perception(format!("Failed to create whisper state: {e}")))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state
            .full(params, &mono_16k)
            .map_err(|e| BrainError::Perception(format!("Whisper transcription failed: {e}")))?;

        let num_segments = state
            .full_n_segments()
            .map_err(|e| BrainError::Perception(format!("Failed to get segment count: {e}")))?;

        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        let text = text.trim().to_string();
        tracing::debug!(transcript = %text, "Transcription complete");
        Ok(text)
    }
}

/// Resample audio to 16 kHz mono f32, as required by Whisper.
fn resample_to_16k_mono(audio: &AudioBuffer) -> Vec<f32> {
    let samples = &audio.samples;
    let channels = audio.channels as usize;
    let source_rate = audio.sample_rate as f64;
    let target_rate = 16_000.0_f64;

    // First, convert to mono by averaging channels.
    let mono: Vec<f32> = if channels > 1 {
        samples
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples.clone()
    };

    // Then resample using linear interpolation.
    if (source_rate - target_rate).abs() < 1.0 {
        return mono;
    }

    let ratio = source_rate / target_rate;
    let output_len = (mono.len() as f64 / ratio) as usize;
    let mut resampled = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < mono.len() {
            mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32
        } else if idx < mono.len() {
            mono[idx]
        } else {
            0.0
        };

        resampled.push(sample);
    }

    resampled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_mono_passthrough() {
        let audio = AudioBuffer {
            samples: vec![0.0, 0.1, 0.2, 0.3],
            sample_rate: 16000,
            channels: 1,
        };
        let result = resample_to_16k_mono(&audio);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn resample_stereo_to_mono() {
        // Stereo: [L0, R0, L1, R1]
        let audio = AudioBuffer {
            samples: vec![0.5, 0.3, 0.7, 0.1],
            sample_rate: 16000,
            channels: 2,
        };
        let result = resample_to_16k_mono(&audio);
        assert_eq!(result.len(), 2);
        assert!((result[0] - 0.4).abs() < 1e-5);
        assert!((result[1] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn resample_downsamples() {
        let audio = AudioBuffer {
            samples: vec![0.0; 48000], // 1 second at 48kHz
            sample_rate: 48000,
            channels: 1,
        };
        let result = resample_to_16k_mono(&audio);
        // Should be approximately 16000 samples
        assert!((result.len() as i64 - 16000).abs() < 2);
    }
}
