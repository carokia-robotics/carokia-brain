//! Sprint 4 demo: persistent memory, semantic recall, and LLM-powered planning.
//!
//! Run with all features:
//!   cargo run --example memory_demo -p carokia-brain --features sqlite,embeddings,llm-planner

#[cfg(feature = "sqlite")]
use carokia_memory::sqlite::SqliteMemory;

#[allow(unused_imports)]
use carokia_memory::{MemoryEntry, MemoryKind, MemoryQuery, MemoryStore};

#[allow(unused_imports)]
use carokia_planner::Goal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Sprint 4: The Mind ===\n");

    // --- Part 1: Persistent Memory (SQLite) ---
    #[cfg(feature = "sqlite")]
    {
        println!("--- Part 1: SQLite Persistent Memory ---");
        let mut mem = SqliteMemory::new(":memory:")?;

        // Store some facts
        let sky = MemoryEntry::new(MemoryKind::Fact, "The sky is blue".into(), 0.9)
            .with_tags(vec!["color".into(), "sky".into()]);
        let owner = MemoryEntry::new(MemoryKind::Fact, "My owner's name is Bob".into(), 1.0)
            .with_tags(vec!["owner".into(), "person".into()]);
        let greeting = MemoryEntry::new(
            MemoryKind::Conversation,
            "User said hello this morning".into(),
            0.5,
        )
        .with_tags(vec!["greeting".into()]);

        mem.store(sky).await?;
        mem.store(owner).await?;
        mem.store(greeting).await?;

        // Recall all
        let all = mem.recall(&MemoryQuery::default()).await?;
        println!("All memories ({}):", all.len());
        for m in &all {
            println!(
                "  [{:?}] {} (importance: {})",
                m.kind, m.content, m.importance
            );
        }

        // Recall by tag
        let color_memories = mem
            .recall(&MemoryQuery {
                tag: Some("color".into()),
                ..Default::default()
            })
            .await?;
        println!("\nMemories tagged 'color' ({}):", color_memories.len());
        for m in &color_memories {
            println!("  {}", m.content);
        }

        // Recall by importance
        let important = mem
            .recall(&MemoryQuery {
                min_importance: Some(0.8),
                ..Default::default()
            })
            .await?;
        println!("\nImportant memories >= 0.8 ({}):", important.len());
        for m in &important {
            println!("  {} ({})", m.content, m.importance);
        }

        println!();
    }

    #[cfg(not(feature = "sqlite"))]
    {
        println!("--- Part 1: SQLite (skipped - enable 'sqlite' feature) ---\n");
    }

    // --- Part 2: Semantic Recall with Mock Embeddings ---
    #[cfg(all(feature = "sqlite", feature = "embeddings"))]
    {
        use carokia_memory::embeddings::{Embedder, MockEmbedder};

        println!("--- Part 2: Semantic Recall ---");
        let mut mem = SqliteMemory::new(":memory:")?;
        let embedder = MockEmbedder;

        let facts = vec![
            ("The sky is blue", vec!["color", "sky"]),
            ("My owner's name is Bob", vec!["owner", "person"]),
            ("Cats are small furry animals", vec!["animals"]),
            ("The kitchen is to the left", vec!["navigation", "rooms"]),
        ];

        for (text, tags) in &facts {
            let entry = MemoryEntry::new(MemoryKind::Fact, text.to_string(), 0.8)
                .with_tags(tags.iter().map(|s| s.to_string()).collect());
            let id = entry.id.clone();
            mem.store(entry).await?;

            let embedding = embedder.embed(text).await?;
            mem.store_embedding(&id, &embedding)?;
        }

        // Semantic query
        let query_text = "What color is the sky?";
        let query_emb = embedder.embed(query_text).await?;
        let results = mem
            .recall(&MemoryQuery {
                query_embedding: Some(query_emb),
                limit: Some(2),
                ..Default::default()
            })
            .await?;

        println!("Semantic search for '{}' (top 2):", query_text);
        for m in &results {
            println!("  {}", m.content);
        }
        println!();
    }

    #[cfg(not(all(feature = "sqlite", feature = "embeddings")))]
    {
        println!(
            "--- Part 2: Semantic Recall (skipped - enable 'sqlite' + 'embeddings' features) ---\n"
        );
    }

    // --- Part 3: LLM-Powered Planning ---
    #[cfg(feature = "llm-planner")]
    {
        use carokia_language::MockBackend;
        use carokia_planner::{LlmPlanner, Planner};
        use std::sync::Arc;

        println!("--- Part 3: LLM-Powered Goal Decomposition ---");

        let mock_response = r#"{"subtasks": [
            {"description": "Check pantry for ingredients", "depends_on": []},
            {"description": "Go to store if missing ingredients", "depends_on": [0]},
            {"description": "Preheat oven to 350F", "depends_on": [1]},
            {"description": "Mix dry ingredients", "depends_on": [1]},
            {"description": "Mix wet ingredients", "depends_on": [1]},
            {"description": "Combine and pour into pan", "depends_on": [3, 4]},
            {"description": "Bake for 30 minutes", "depends_on": [2, 5]}
        ]}"#;

        let backend = Arc::new(MockBackend::new(mock_response));
        let planner = LlmPlanner::new(backend);
        let goal = Goal::new("Bake a chocolate cake for the owner", 7);

        let tasks = planner.decompose(&goal).await?;
        println!("Goal: {}", goal.description);
        println!("Decomposed into {} subtasks:", tasks.len());
        for (i, task) in tasks.iter().enumerate() {
            let deps = if task.depends_on.is_empty() {
                String::from("none")
            } else {
                task.depends_on
                    .iter()
                    .filter_map(|dep_id| {
                        tasks
                            .iter()
                            .position(|t| &t.id == dep_id)
                            .map(|idx| format!("#{}", idx))
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            println!("  #{}: {} (depends on: {})", i, task.description, deps);
        }
        println!();
    }

    #[cfg(not(feature = "llm-planner"))]
    {
        println!(
            "--- Part 3: LLM Planning (skipped - enable 'llm-planner' feature) ---\n"
        );
    }

    println!("=== Demo complete ===");
    Ok(())
}
