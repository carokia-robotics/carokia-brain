//! Camera capture abstractions for the vision system.
//!
//! Provides a `CameraSource` trait with two implementations:
//! - `FfmpegCamera`: captures frames via ffmpeg subprocess (macOS AVFoundation)
//! - `FileCamera`: loads frames from a file on disk (for testing / offline use)

use async_trait::async_trait;
use carokia_core::BrainError;

/// Trait for capturing image frames from a camera or camera-like source.
#[async_trait]
pub trait CameraSource: Send + Sync {
    /// Capture a single frame and return it as JPEG bytes.
    async fn capture_frame(&self) -> Result<Vec<u8>, BrainError>;
}

/// Captures frames from a camera using ffmpeg as a subprocess.
///
/// On macOS this uses the AVFoundation input device. On Linux it would
/// use v4l2 (adjustable via the `input_format` field).
pub struct FfmpegCamera {
    device_index: u32,
    width: u32,
    height: u32,
    input_format: String,
}

impl FfmpegCamera {
    /// Create a new ffmpeg-based camera source.
    ///
    /// `device_index` is the AVFoundation device index (typically 0 for the
    /// built-in webcam on macOS).
    pub fn new(device_index: u32) -> Self {
        Self {
            device_index,
            width: 640,
            height: 480,
            input_format: "avfoundation".to_string(),
        }
    }

    /// Set the capture resolution.
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the input format (e.g. "avfoundation" for macOS, "v4l2" for Linux).
    pub fn with_input_format(mut self, format: impl Into<String>) -> Self {
        self.input_format = format.into();
        self
    }
}

#[async_trait]
impl CameraSource for FfmpegCamera {
    async fn capture_frame(&self) -> Result<Vec<u8>, BrainError> {
        let video_size = format!("{}x{}", self.width, self.height);
        let device = format!("{}:none", self.device_index);

        let output = tokio::process::Command::new("ffmpeg")
            .args([
                "-f",
                &self.input_format,
                "-video_size",
                &video_size,
                "-framerate",
                "1",
                "-i",
                &device,
                "-frames:v",
                "1",
                "-f",
                "image2pipe",
                "-vcodec",
                "mjpeg",
                "-",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| BrainError::Sensor(format!("ffmpeg launch error: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BrainError::Sensor(format!(
                "ffmpeg capture failed (exit {}): {}",
                output.status,
                stderr.chars().take(200).collect::<String>()
            )));
        }

        if output.stdout.is_empty() {
            return Err(BrainError::Sensor("ffmpeg produced empty output".into()));
        }

        Ok(output.stdout)
    }
}

/// Loads frames from a file on disk. Useful for testing and offline analysis.
pub struct FileCamera {
    path: std::path::PathBuf,
}

impl FileCamera {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl CameraSource for FileCamera {
    async fn capture_frame(&self) -> Result<Vec<u8>, BrainError> {
        tokio::fs::read(&self.path)
            .await
            .map_err(|e| BrainError::Sensor(format!("file camera read error: {e}")))
    }
}

/// An in-memory camera source for unit testing.
pub struct MemoryCamera {
    data: Vec<u8>,
}

impl MemoryCamera {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

#[async_trait]
impl CameraSource for MemoryCamera {
    async fn capture_frame(&self) -> Result<Vec<u8>, BrainError> {
        Ok(self.data.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_camera_returns_data() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0]; // JPEG magic bytes
        let cam = MemoryCamera::new(data.clone());
        let frame = cam.capture_frame().await.unwrap();
        assert_eq!(frame, data);
    }

    #[tokio::test]
    async fn file_camera_error_on_missing_file() {
        let cam = FileCamera::new("/nonexistent/path/image.jpg");
        let result = cam.capture_frame().await;
        assert!(result.is_err());
    }

    #[test]
    fn ffmpeg_camera_builder() {
        let cam = FfmpegCamera::new(0)
            .with_resolution(1280, 720)
            .with_input_format("v4l2");
        assert_eq!(cam.width, 1280);
        assert_eq!(cam.height, 720);
        assert_eq!(cam.input_format, "v4l2");
    }
}
