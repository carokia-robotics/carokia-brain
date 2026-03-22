use carokia_language::{
    build_personality_prompt,
    config::{LlmProviderConfig, PersonalityConfig},
    create_backend, ConversationManager,
};
use std::io::{self, BufRead, Write};

#[cfg(feature = "cli")]
use clap::Parser;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "carokia-chat", about = "Chat with Carokia AI")]
struct Args {
    /// LLM model name
    #[arg(long, default_value = "gemma3:latest")]
    model: String,
    /// Ollama host URL
    #[arg(long, default_value = "http://localhost")]
    host: String,
    /// Ollama port
    #[arg(long, default_value_t = 11434)]
    port: u16,
    /// Enable verbose logging
    #[arg(long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "cli")]
    let args = Args::parse();

    #[cfg(feature = "cli")]
    {
        let level = if args.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        };
        tracing_subscriber::fmt().with_max_level(level).init();
    }

    #[cfg(not(feature = "cli"))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let personality = PersonalityConfig::default();

    println!("\u{2554}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2557}");
    println!(
        "\u{2551}     {} \u{2014} AI Companion        \u{2551}",
        personality.name
    );
    println!("\u{2551}     Type 'quit' to exit           \u{2551}");
    println!("\u{255a}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{2550}\u{255d}");
    println!();

    // Build config from CLI args or defaults
    #[cfg(feature = "cli")]
    let config = LlmProviderConfig::Ollama {
        host: args.host,
        port: args.port,
        model: args.model,
    };

    #[cfg(not(feature = "cli"))]
    let config = LlmProviderConfig::default();

    let backend = create_backend(&config);

    let system_prompt = build_personality_prompt(&personality);
    let mut conversation = ConversationManager::with_system_prompt(backend, 20, system_prompt);

    let stdin = io::stdin();
    loop {
        print!("\nYou: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        stdin.lock().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }
        if input == "quit" || input == "exit" {
            break;
        }

        match conversation.chat(input).await {
            Ok(response) => println!("\n{}: {response}", personality.name),
            Err(e) => eprintln!("\n[Error: {e}]"),
        }
    }

    println!("\nGoodbye!");
}
