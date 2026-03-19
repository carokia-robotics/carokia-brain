use std::time::Duration;

use carokia_brain::{Brain, BrainConfig};
use carokia_planner::Goal;
use tokio_util::sync::CancellationToken;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Carokia Brain Demo ===");

    let config = BrainConfig {
        tick_rate_hz: 2.0,
        memory_capacity: 50,
        conversation_history_limit: 20,
    };

    let mut brain = Brain::new(config);

    // Add some goals.
    brain.add_goal(Goal::new("Navigate to the charging station", 8));
    brain.add_goal(Goal::new("Say hello to nearby humans", 3));

    // Run a few manual ticks to show output.
    println!("\n--- Running 5 manual ticks ---\n");
    for i in 1..=5 {
        match brain.tick().await {
            Ok(Some(action)) => {
                println!("Tick {i}: Action => {action:?}");
            }
            Ok(None) => {
                println!("Tick {i}: No action");
            }
            Err(e) => {
                eprintln!("Tick {i}: Error => {e}");
            }
        }
    }

    // Demonstrate the run loop with cancellation.
    println!("\n--- Running autonomous loop for 2 seconds ---\n");
    let cancel = CancellationToken::new();
    let cancel2 = cancel.clone();

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        cancel2.cancel();
    });

    if let Err(e) = brain.run(cancel).await {
        eprintln!("Brain loop error: {e}");
    }

    println!("\n=== Demo complete ===");
}
