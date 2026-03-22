//! LLM-powered face detection using Ollama vision models.
//!
//! Uses a vision-capable model to detect and describe human faces in images.
//! This provides richer information than traditional Haar cascades (expression,
//! approximate age, etc.) at the cost of higher latency.

use crate::vision::VisionAnalyzer;
use carokia_core::BrainError;
use serde::{Deserialize, Serialize};

/// Description of a single detected face.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDescription {
    /// Free-text description of the face (age, expression, etc.).
    pub description: String,
    /// Detected facial expression (e.g. "smiling", "neutral", "surprised").
    pub expression: String,
}

/// Result of face detection on an image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceResult {
    /// Number of faces detected.
    pub count: usize,
    /// Per-face descriptions.
    pub faces: Vec<FaceDescription>,
    /// Raw response from the vision model.
    pub raw_response: String,
}

/// Detects faces in images using an Ollama vision model.
pub struct FaceDetector<'a> {
    vision: &'a VisionAnalyzer,
}

impl<'a> FaceDetector<'a> {
    pub fn new(vision: &'a VisionAnalyzer) -> Self {
        Self { vision }
    }

    /// Detect and describe faces in an image.
    pub async fn detect_faces(&self, image_bytes: &[u8]) -> Result<FaceResult, BrainError> {
        let prompt = concat!(
            "How many human faces do you see in this image? ",
            "For each face, describe the approximate age, expression, and any notable features. ",
            "If no faces are visible, say 'No faces detected.' ",
            "Format: start with 'Count: N' on the first line, then describe each face on its own line ",
            "starting with 'Face 1:', 'Face 2:', etc. For each face include 'Expression: <word>' somewhere in the line."
        );

        let raw_response = self
            .vision
            .analyze_with_prompt(image_bytes, prompt)
            .await?;

        let (count, faces) = parse_face_response(&raw_response);

        Ok(FaceResult {
            count,
            faces,
            raw_response,
        })
    }
}

/// Parse the LLM response into structured face data.
fn parse_face_response(response: &str) -> (usize, Vec<FaceDescription>) {
    let mut count = 0;
    let mut faces = Vec::new();

    for line in response.lines() {
        let trimmed = line.trim();

        // Parse count line.
        if let Some(rest) = trimmed.strip_prefix("Count:") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                count = n;
            }
        }

        // Parse face description lines.
        if trimmed.starts_with("Face ") && trimmed.contains(':') {
            let description = trimmed
                .split_once(':')
                .map(|(_, rest)| rest)
                .unwrap_or("")
                .trim()
                .to_string();

            let expression = extract_expression(trimmed);

            faces.push(FaceDescription {
                description,
                expression,
            });
        }
    }

    // If we parsed faces but no explicit count, use the face count.
    if count == 0 && !faces.is_empty() {
        count = faces.len();
    }

    // Check for "no faces" responses.
    let lower = response.to_lowercase();
    if lower.contains("no faces") || lower.contains("0 faces") || lower.contains("no human faces")
    {
        count = 0;
        faces.clear();
    }

    (count, faces)
}

/// Extract the expression keyword from a face description line.
fn extract_expression(line: &str) -> String {
    let lower = line.to_lowercase();
    if let Some(idx) = lower.find("expression:") {
        let after = &line[idx + "expression:".len()..];
        let word = after
            .trim()
            .split(|c: char| c == ',' || c == '.' || c == ';' || c.is_whitespace())
            .next()
            .unwrap_or("unknown");
        return word.trim().to_string();
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_face_response_with_faces() {
        let response = "\
Count: 2
Face 1: Young adult, approximately 25 years old. Expression: smiling, looking at camera.
Face 2: Middle-aged person, approximately 45 years old. Expression: neutral, looking away.";

        let (count, faces) = parse_face_response(response);
        assert_eq!(count, 2);
        assert_eq!(faces.len(), 2);
        assert_eq!(faces[0].expression, "smiling");
        assert_eq!(faces[1].expression, "neutral");
    }

    #[test]
    fn parse_face_response_no_faces() {
        let response = "No faces detected. The image shows an empty room.";
        let (count, faces) = parse_face_response(response);
        assert_eq!(count, 0);
        assert!(faces.is_empty());
    }

    #[test]
    fn parse_face_response_infers_count() {
        let response = "\
Face 1: A person smiling. Expression: happy
Face 2: A person frowning. Expression: sad";

        let (count, faces) = parse_face_response(response);
        assert_eq!(count, 2);
        assert_eq!(faces.len(), 2);
    }

    #[test]
    fn extract_expression_works() {
        assert_eq!(
            extract_expression("Face 1: young adult, Expression: smiling, cheerful"),
            "smiling"
        );
        assert_eq!(
            extract_expression("Face 1: no expression tag here"),
            "unknown"
        );
    }

    #[test]
    fn face_result_serialization() {
        let result = FaceResult {
            count: 1,
            faces: vec![FaceDescription {
                description: "A person".to_string(),
                expression: "neutral".to_string(),
            }],
            raw_response: "Count: 1\nFace 1: A person. Expression: neutral".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: FaceResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.count, 1);
        assert_eq!(parsed.faces[0].expression, "neutral");
    }
}
