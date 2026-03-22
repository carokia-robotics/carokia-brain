//! Conversationalist demo: streaming output, tool use, personality.
//!
//! Run with: cargo run --example conversationalist_demo

use carokia_language::{
    build_personality_prompt,
    config::PersonalityConfig,
    tools::{self, CalculatorTool, CurrentTimeTool, ToolRegistry},
    ConversationManager, MockBackend,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // 1. Personality
    let personality = PersonalityConfig {
        name: "Carokia".to_string(),
        traits: vec![
            "helpful".to_string(),
            "curious".to_string(),
            "witty".to_string(),
        ],
        speaking_style: "warm, concise, with a touch of humor".to_string(),
        backstory: "An autonomous robot companion exploring the world.".to_string(),
        response_length: "moderate".to_string(),
    };

    let system_prompt = build_personality_prompt(&personality);
    println!("=== Personality System Prompt ===");
    println!("{system_prompt}");
    println!();

    // 2. Tool use
    println!("=== Tool Use Demo ===");
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CurrentTimeTool));
    registry.register(Box::new(CalculatorTool));

    println!("Available tools:");
    println!("{}", registry.list_descriptions());
    println!();

    // Ask the time
    let time_result = registry.execute("current_time", "").await.unwrap();
    println!("Current time: {time_result}");

    // Do math
    let math_result = registry.execute("calculator", "42 * 13 + 7").await.unwrap();
    println!("42 * 13 + 7 = {math_result}");

    let math_result2 = registry
        .execute("calculator", "(100 - 37) / 9")
        .await
        .unwrap();
    println!("(100 - 37) / 9 = {math_result2}");
    println!();

    // 3. Streaming output
    println!("=== Streaming Demo ===");
    #[cfg(feature = "streaming")]
    {
        use carokia_language::StreamingLlmBackend;
        use std::io::Write;

        let backend = MockBackend::new(
            "Hello! I am Carokia, your autonomous robot companion. \
             I can help you with many things, from answering questions \
             to performing calculations. How can I assist you today?",
        );

        let params = carokia_language::GenerateParams::default();
        let mut rx = backend.generate_stream("Hi there!", &params).await.unwrap();

        print!("{}: ", personality.name);
        while let Some(token_result) = rx.recv().await {
            match token_result {
                Ok(token) => {
                    print!("{token}");
                    std::io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("\n[Stream error: {e}]");
                    break;
                }
            }
        }
        println!();
    }
    #[cfg(not(feature = "streaming"))]
    {
        println!("(Streaming feature not enabled, using regular generation)");
    }
    println!();

    // 4. Memory-augmented conversation
    println!("=== Memory-Augmented Conversation Demo ===");
    let backend = MockBackend::new("Based on what I know about you, you enjoy hiking and photography! Those are great hobbies.");
    let mut conversation =
        ConversationManager::with_system_prompt(Box::new(backend), 20, system_prompt);

    // Inject memory context
    conversation.set_context(
        "Relevant memories:\n- User enjoys hiking in the mountains\n- User is learning photography\n- User's favorite color is blue".to_string(),
    );

    let reply = conversation
        .chat("What do you know about my hobbies?")
        .await
        .unwrap();
    println!("{}: {reply}", personality.name);
    println!();

    // 5. Full tool registry with defaults
    println!("=== Default Tool Registry ===");
    let default_reg = tools::default_tools();
    println!("{}", default_reg.list_descriptions());

    println!("\nDemo complete!");
}
