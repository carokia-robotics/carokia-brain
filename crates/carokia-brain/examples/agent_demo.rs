//! Agent demo — chain-of-thought reasoning, emotional state, and self-reflection.
//!
//! Run with: cargo run --example agent_demo -p carokia-brain --features "reasoning,reflection"
//!
//! This demo shows:
//! - ChainOfThoughtReasoner working through a multi-step question
//! - Emotional state tracking across events
//! - Self-reflection generating insights from recent memories

#[cfg(all(feature = "reasoning", feature = "reflection"))]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use carokia_core::emotion::{EmotionalEvent, EmotionalState};
    use carokia_language::MockBackend;
    use carokia_memory::reflection::ReflectionEngine;
    use carokia_memory::{MemoryEntry, MemoryKind};
    use carokia_planner::reasoning::ChainOfThoughtReasoner;

    println!("=== Carokia Agent Demo ===\n");

    // --- 1. Chain-of-thought reasoning ---
    println!("--- Chain-of-Thought Reasoning ---\n");

    let mock_reasoning = Arc::new(MockBackend::new(
        "Thought: To answer this I need to consider what makes a robot helpful.\n\
         Thought: A helpful robot must understand context, learn from experience, and communicate clearly.\n\
         Observation: These are the core principles of assistive robotics.\n\
         Answer: A robot is most helpful when it combines contextual understanding with empathetic communication."
    ));

    let reasoner = ChainOfThoughtReasoner::new(mock_reasoning, 10);
    let chain = reasoner
        .reason("What makes a robot truly helpful to humans?")
        .await
        .expect("reasoning failed");

    for (i, step) in chain.steps.iter().enumerate() {
        println!("  Step {}: [{:?}] {}", i + 1, step.step_type, step.content);
    }

    if let Some(ref answer) = chain.final_answer {
        println!("\n  Final Answer: {answer}");
    }

    // --- 2. Emotional state ---
    println!("\n--- Emotional State Tracking ---\n");

    let mut emotion = EmotionalState::neutral();
    println!(
        "  Initial: {} ({})",
        emotion.mood_label(),
        emotion.to_prompt_modifier()
    );

    let events = [
        ("Goal completed", EmotionalEvent::GoalCompleted),
        ("Positive interaction", EmotionalEvent::PositiveInteraction),
        ("Threat detected", EmotionalEvent::ThreatDetected),
        ("Time passes (decay)", EmotionalEvent::Idle),
    ];

    for (label, event) in events {
        emotion.update(event);
        println!(
            "  After {label}: {} (valence={:.2}, arousal={:.2})",
            emotion.mood_label(),
            emotion.valence,
            emotion.arousal
        );
    }

    // Apply some decay
    for _ in 0..5 {
        emotion.decay(1.0);
    }
    println!(
        "  After 5s decay: {} (valence={:.2}, arousal={:.2})",
        emotion.mood_label(),
        emotion.valence,
        emotion.arousal
    );

    // --- 3. Self-reflection ---
    println!("\n--- Self-Reflection ---\n");

    let mock_reflection = Arc::new(MockBackend::new(
        "I notice a recurring pattern of obstacle encounters followed by avoidance maneuvers, \
         suggesting the current patrol route may need optimization.",
    ));

    let reflection_engine = ReflectionEngine::new(mock_reflection, 5);

    let memories = vec![
        MemoryEntry::new(
            MemoryKind::Perception,
            "Obstacle detected at 2m".into(),
            0.5,
        ),
        MemoryEntry::new(
            MemoryKind::Event,
            "Avoided obstacle successfully".into(),
            0.7,
        ),
        MemoryEntry::new(
            MemoryKind::Perception,
            "Obstacle detected at 1.5m".into(),
            0.6,
        ),
        MemoryEntry::new(
            MemoryKind::Event,
            "Avoided obstacle successfully".into(),
            0.7,
        ),
        MemoryEntry::new(
            MemoryKind::Conversation,
            "User asked about patrol status".into(),
            0.4,
        ),
    ];

    if reflection_engine.should_reflect(memories.len()) {
        let insight = reflection_engine
            .reflect(&memories)
            .await
            .expect("reflection failed");
        println!("  Reflection insight: {}", insight.content);
        println!("  Tags: {:?}", insight.tags);
    }

    println!("\n=== Demo Complete ===");
}

#[cfg(not(all(feature = "reasoning", feature = "reflection")))]
fn main() {
    println!("This example requires the 'reasoning' and 'reflection' features.");
    println!("Run with: cargo run --example agent_demo -p carokia-brain --features \"reasoning,reflection\"");
}
