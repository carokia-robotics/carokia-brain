//! Vision demo: captures frames and analyzes them with an LLM vision model.
//!
//! Usage:
//!   # Live camera (requires ffmpeg):
//!   cargo run --example vision_demo --features vision
//!
//!   # From an image file:
//!   cargo run --example vision_demo --features vision -- --file path/to/image.jpg
//!
//!   # Single shot (no loop):
//!   cargo run --example vision_demo --features vision -- --file image.jpg --once

#[cfg(feature = "vision")]
mod inner {
    use carokia_perception::camera::{CameraSource, FfmpegCamera, FileCamera};
    use carokia_perception::face::FaceDetector;
    use carokia_perception::vision::{VisionAnalyzer, VisionConfig};
    use std::time::Duration;

    pub async fn run() {
        let args: Vec<String> = std::env::args().collect();

        let file_path = args
            .iter()
            .position(|a| a == "--file")
            .and_then(|i| args.get(i + 1))
            .cloned();

        let once = args.iter().any(|a| a == "--once");

        let model = args
            .iter()
            .position(|a| a == "--model")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| "llava".to_string());

        let config = VisionConfig {
            model,
            ..VisionConfig::default()
        };

        println!("=== Carokia Vision Demo ===");
        println!("Model: {}", config.model);

        let vision = VisionAnalyzer::new(config);
        let face_detector = FaceDetector::new(&vision);

        let camera: Box<dyn CameraSource> = if let Some(ref path) = file_path {
            println!("Source: file ({})", path);
            Box::new(FileCamera::new(path))
        } else {
            println!("Source: live camera (ffmpeg)");
            Box::new(FfmpegCamera::new(0))
        };

        loop {
            println!("\n--- Capturing frame ---");
            match camera.capture_frame().await {
                Ok(frame) => {
                    println!("Frame captured: {} bytes", frame.len());

                    // Scene analysis.
                    println!("\nAnalyzing scene...");
                    match vision.analyze(&frame).await {
                        Ok(result) => {
                            println!("Description: {}", result.description);
                            if !result.objects.is_empty() {
                                println!("Objects: {}", result.objects.join(", "));
                            }
                        }
                        Err(e) => {
                            eprintln!("Scene analysis error: {e}");
                        }
                    }

                    // Face detection.
                    println!("\nDetecting faces...");
                    match face_detector.detect_faces(&frame).await {
                        Ok(result) => {
                            println!("Faces detected: {}", result.count);
                            for (i, face) in result.faces.iter().enumerate() {
                                println!(
                                    "  Face {}: {} (expression: {})",
                                    i + 1,
                                    face.description,
                                    face.expression
                                );
                            }
                        }
                        Err(e) => {
                            eprintln!("Face detection error: {e}");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Capture error: {e}");
                }
            }

            if once {
                break;
            }

            println!("\nWaiting 5 seconds before next capture...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }

        println!("\n=== Demo complete ===");
    }
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "vision")]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
        inner::run().await;
    }

    #[cfg(not(feature = "vision"))]
    {
        eprintln!("This example requires the 'vision' feature.");
        eprintln!("Run with: cargo run --example vision_demo --features vision");
        std::process::exit(1);
    }
}
