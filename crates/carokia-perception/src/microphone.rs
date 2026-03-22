//! Microphone audio capture via cpal.
//!
//! Gated behind the `audio` feature flag.

use carokia_core::{AudioBuffer, BrainError};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Captures audio from the system default input device.
pub struct MicrophoneSource {
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
}

impl MicrophoneSource {
    /// Create a new MicrophoneSource using the default input device.
    pub fn new() -> Result<Self, BrainError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| BrainError::Sensor("No default input device found".into()))?;

        let config = device
            .default_input_config()
            .map_err(|e| BrainError::Sensor(format!("Failed to get input config: {e}")))?;

        tracing::info!(
            device = device.name().unwrap_or_default(),
            sample_rate = config.sample_rate().0,
            channels = config.channels(),
            "Microphone initialized"
        );

        Ok(Self { device, config })
    }

    /// Record audio for the given duration and return an AudioBuffer.
    pub fn record_chunk(&self, duration_secs: f32) -> Result<AudioBuffer, BrainError> {
        let sample_rate = self.config.sample_rate().0;
        let channels = self.config.channels();
        let total_samples = (sample_rate as f32 * duration_secs * channels as f32) as usize;

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::with_capacity(total_samples)));
        let buffer_clone = Arc::clone(&buffer);
        let done = Arc::new(Mutex::new(false));
        let done_clone = Arc::clone(&done);

        let err_fn = |err: cpal::StreamError| {
            tracing::error!("Audio stream error: {err}");
        };

        let stream = match self.config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config = self.config.clone().into();
                self.device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut buf = buffer_clone.lock().unwrap();
                        if buf.len() < total_samples {
                            let remaining = total_samples - buf.len();
                            let to_copy = data.len().min(remaining);
                            buf.extend_from_slice(&data[..to_copy]);
                            if buf.len() >= total_samples {
                                *done_clone.lock().unwrap() = true;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            cpal::SampleFormat::I16 => {
                let config = self.config.clone().into();
                let buf = Arc::clone(&buffer);
                let d = Arc::clone(&done);
                self.device.build_input_stream(
                    &config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut buf = buf.lock().unwrap();
                        if buf.len() < total_samples {
                            let remaining = total_samples - buf.len();
                            let to_copy = data.len().min(remaining);
                            for &sample in &data[..to_copy] {
                                buf.push(sample as f32 / i16::MAX as f32);
                            }
                            if buf.len() >= total_samples {
                                *d.lock().unwrap() = true;
                            }
                        }
                    },
                    err_fn,
                    None,
                )
            }
            format => {
                return Err(BrainError::Sensor(format!(
                    "Unsupported sample format: {format:?}"
                )));
            }
        }
        .map_err(|e| BrainError::Sensor(format!("Failed to build input stream: {e}")))?;

        stream
            .play()
            .map_err(|e| BrainError::Sensor(format!("Failed to start recording: {e}")))?;

        tracing::info!(duration_secs, "Recording audio...");

        // Block until we have enough samples or timeout.
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs_f32(duration_secs + 1.0);
        loop {
            if *done.lock().unwrap() {
                break;
            }
            if start.elapsed() > timeout {
                tracing::warn!("Recording timed out");
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        drop(stream);

        let samples = Arc::try_unwrap(buffer)
            .unwrap_or_else(|arc| arc.lock().unwrap().clone())
            .into_inner()
            .unwrap();

        tracing::info!(samples = samples.len(), "Recording complete");

        Ok(AudioBuffer {
            samples,
            sample_rate,
            channels,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires a real microphone device
    fn microphone_source_can_be_created() {
        let mic = MicrophoneSource::new();
        assert!(mic.is_ok());
    }

    #[test]
    #[ignore] // Requires a real microphone device
    fn microphone_records_audio() {
        let mic = MicrophoneSource::new().unwrap();
        let buffer = mic.record_chunk(1.0).unwrap();
        assert!(!buffer.samples.is_empty());
        assert!(buffer.sample_rate > 0);
    }
}
