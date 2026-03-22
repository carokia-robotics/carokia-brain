//! Voice conversation demo for Carokia.
//!
//! Requires the `voice` feature on carokia-brain, which enables audio capture
//! and Whisper STT. Also requires a Whisper model file.
//!
//! # Setup
//! ```sh
//! mkdir -p models
//! curl -L -o models/ggml-base.bin \
//!   https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
//! ```
//!
//! # Run
//! ```sh
//! cargo run --example voice_chat -p carokia-brain --features carokia-brain/voice
//! ```

#[cfg(feature = "voice")]
use carokia_language::tts::{SystemTts, TextToSpeech};
#[cfg(feature = "voice")]
use carokia_language::{config::LlmProviderConfig, create_backend, ConversationManager};
#[cfg(feature = "voice")]
use carokia_perception::microphone::MicrophoneSource;
#[cfg(feature = "voice")]
use carokia_perception::whisper::WhisperTranscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    #[cfg(not(feature = "voice"))]
    {
        eprintln!("Voice chat requires the 'voice' feature.");
        eprintln!();
        eprintln!("Run with:");
        eprintln!("  cargo run --example voice_chat -p carokia-brain --features carokia-brain/voice");
        std::process::exit(1);
    }

    #[cfg(feature = "voice")]
    run_voice_chat().await;
}

#[cfg(feature = "voice")]
async fn run_voice_chat() {
    println!("╔═══════════════════════════════════════╗");
    println!("║     CAROKIA — Voice Chat              ║");
    println!("║     Press Ctrl+C to exit              ║");
    println!("╚═══════════════════════════════════════╝");
    println!();

    // Determine model path.
    let model_path =
        std::env::var("WHISPER_MODEL").unwrap_or_else(|_| "models/ggml-base.bin".to_string());

    let record_seconds: f32 = std::env::var("RECORD_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5.0);

    // Initialize microphone.
    println!("Initializing microphone...");
    let mic = MicrophoneSource::new().expect("Failed to initialize microphone");

    // Load Whisper model.
    println!("Loading Whisper model from {model_path}...");
    let whisper = WhisperTranscriber::new(&model_path).expect("Failed to load Whisper model");

    // Initialize TTS.
    let tts = SystemTts::new();

    // Initialize LLM conversation.
    let config = LlmProviderConfig::default();
    let backend = create_backend(&config);
    let system_prompt = "You are Carokia, an advanced autonomous robot companion. \
        You are helpful, protective, and loyal. Keep responses concise — \
        you are speaking aloud, so be brief and natural.";
    let mut conversation =
        ConversationManager::with_system_prompt(backend, 20, system_prompt.to_string());

    println!("\nReady! Speak into your microphone.\n");

    // Set up Ctrl+C handler.
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    let _ = ctrlc::set_handler(move || {
        println!("\nShutting down...");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    });

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        println!("Listening ({record_seconds}s)...");

        // Record audio.
        let audio = match mic.record_chunk(record_seconds) {
            Ok(buf) => buf,
            Err(e) => {
                eprintln!("[Recording error: {e}]");
                continue;
            }
        };

        // Transcribe.
        print!("Transcribing... ");
        let transcript = match whisper.transcribe(&audio) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("[Transcription error: {e}]");
                continue;
            }
        };

        if transcript.is_empty() {
            println!("(silence)");
            continue;
        }

        println!("\nYou said: {transcript}");

        // Chat with LLM.
        match conversation.chat(&transcript).await {
            Ok(response) => {
                println!("\nCarokia: {response}\n");

                // Speak the response.
                if let Err(e) = tts.speak(&response).await {
                    eprintln!("[TTS error: {e}]");
                }
            }
            Err(e) => {
                eprintln!("[Chat error: {e}]");
            }
        }
    }

    println!("\nGoodbye!");
}
