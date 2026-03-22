use carokia_language::{config::LlmProviderConfig, create_backend, ConversationManager};
use std::io::{self, BufRead, Write};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("╔═══════════════════════════════════╗");
    println!("║     CAROKIA — AI Companion        ║");
    println!("║     Type 'quit' to exit           ║");
    println!("╚═══════════════════════════════════╝");
    println!();

    // Load config or use defaults
    let config = LlmProviderConfig::default();
    let backend = create_backend(&config);

    let system_prompt = "You are Carokia, an advanced autonomous robot companion. You are helpful, protective, and loyal. Keep responses concise.";
    let mut conversation =
        ConversationManager::with_system_prompt(backend, 20, system_prompt.to_string());

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
            Ok(response) => println!("\nCarokia: {response}"),
            Err(e) => eprintln!("\n[Error: {e}]"),
        }
    }

    println!("\nGoodbye!");
}
