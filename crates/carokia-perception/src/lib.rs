#[cfg(feature = "audio")]
pub mod microphone;

#[cfg(feature = "whisper")]
pub mod whisper;

#[cfg(feature = "vision")]
pub mod camera;

#[cfg(feature = "vision")]
pub mod vision;

#[cfg(feature = "vision")]
pub mod face;

use async_trait::async_trait;
use carokia_core::{BrainError, Modality, SensorFrame, Timestamp};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Content produced by perception processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerceptContent {
    ObjectDetection {
        label: String,
        confidence: f64,
        bbox: [f64; 4],
    },
    AudioEvent {
        kind: String,
    },
    SpeechTranscript {
        text: String,
    },
    Obstacle {
        distance: f64,
        bearing: f64,
    },
    SceneDescription {
        description: String,
        objects: Vec<String>,
    },
    FaceDetection {
        count: usize,
        descriptions: Vec<String>,
    },
    Person {
        name: Option<String>,
        distance: f64,
        bearing: f64,
    },
}

/// A processed percept from sensor data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percept {
    pub id: String,
    pub timestamp: Timestamp,
    pub source_modality: Modality,
    pub content: PerceptContent,
    pub confidence: f64,
}

impl Percept {
    pub fn new(modality: Modality, content: PerceptContent, confidence: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Timestamp::now(),
            source_modality: modality,
            content,
            confidence,
        }
    }
}

/// Processes sensor frames into percepts.
#[async_trait]
pub trait PerceptionProcessor: Send + Sync {
    fn supported_modality(&self) -> Modality;
    async fn process(&self, frame: &SensorFrame) -> Result<Vec<Percept>, BrainError>;
}

/// Routes sensor frames to the appropriate processors.
pub struct PerceptionPipeline {
    processors: Vec<Box<dyn PerceptionProcessor>>,
}

impl PerceptionPipeline {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add_processor(&mut self, processor: Box<dyn PerceptionProcessor>) {
        self.processors.push(processor);
    }

    pub async fn process_frame(&self, frame: &SensorFrame) -> Result<Vec<Percept>, BrainError> {
        let mut percepts = Vec::new();
        for proc in &self.processors {
            if proc.supported_modality() == frame.modality {
                let mut results = proc.process(frame).await?;
                percepts.append(&mut results);
            }
        }
        Ok(percepts)
    }
}

impl Default for PerceptionPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// A stub processor that generates dummy percepts for testing.
pub struct StubProcessor {
    modality: Modality,
}

impl StubProcessor {
    pub fn new(modality: Modality) -> Self {
        Self { modality }
    }
}

#[async_trait]
impl PerceptionProcessor for StubProcessor {
    fn supported_modality(&self) -> Modality {
        self.modality
    }

    async fn process(&self, frame: &SensorFrame) -> Result<Vec<Percept>, BrainError> {
        let content = match frame.modality {
            Modality::Vision => PerceptContent::ObjectDetection {
                label: "unknown_object".to_string(),
                confidence: 0.5,
                bbox: [0.0, 0.0, 1.0, 1.0],
            },
            Modality::Audio => PerceptContent::AudioEvent {
                kind: "ambient_noise".to_string(),
            },
            Modality::Lidar => PerceptContent::Obstacle {
                distance: 2.0,
                bearing: 0.0,
            },
            _ => PerceptContent::AudioEvent {
                kind: "stub".to_string(),
            },
        };

        let percept = Percept::new(frame.modality, content, 0.5);
        Ok(vec![percept])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_core::SensorPayload;

    #[tokio::test]
    async fn stub_processor_produces_percepts() {
        let proc = StubProcessor::new(Modality::Vision);
        let frame = SensorFrame::new(Modality::Vision, SensorPayload::Text("img".into()));
        let percepts = proc.process(&frame).await.unwrap();
        assert_eq!(percepts.len(), 1);
        assert_eq!(percepts[0].source_modality, Modality::Vision);
    }

    #[tokio::test]
    async fn pipeline_routes_to_matching_processor() {
        let mut pipeline = PerceptionPipeline::new();
        pipeline.add_processor(Box::new(StubProcessor::new(Modality::Vision)));
        pipeline.add_processor(Box::new(StubProcessor::new(Modality::Lidar)));

        let vision_frame = SensorFrame::new(Modality::Vision, SensorPayload::Text("img".into()));
        let percepts = pipeline.process_frame(&vision_frame).await.unwrap();
        assert_eq!(percepts.len(), 1);

        let lidar_frame = SensorFrame::new(Modality::Lidar, SensorPayload::Bytes(vec![0]));
        let percepts = pipeline.process_frame(&lidar_frame).await.unwrap();
        assert_eq!(percepts.len(), 1);
    }

    #[tokio::test]
    async fn pipeline_skips_unmatched_modality() {
        let mut pipeline = PerceptionPipeline::new();
        pipeline.add_processor(Box::new(StubProcessor::new(Modality::Vision)));

        let audio_frame = SensorFrame::new(Modality::Audio, SensorPayload::Text("beep".into()));
        let percepts = pipeline.process_frame(&audio_frame).await.unwrap();
        assert!(percepts.is_empty());
    }
}
