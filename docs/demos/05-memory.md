# Demo Script: Memory and Planning

## Overview
Demonstrate Carokia's persistent memory (SQLite-backed) and LLM-powered planning. The robot remembers facts across sessions and can decompose high-level goals into step-by-step plans.

## Prerequisites
- Ollama running with a chat model

## Commands

```bash
cargo run --example memory_demo --features sqlite,llm-planner
```

## Walkthrough

1. **Launch** -- the demo initializes an SQLite database (created in a temp directory or the project root).

2. **Memory storage** -- the demo stores several facts (e.g., operator name, mission parameters).
   - Expected: Each fact is printed as it is stored, confirming persistence.

3. **Memory recall** -- the demo queries stored facts.
   - Expected: Previously stored facts are retrieved accurately, demonstrating the persistence layer.

4. **Planning** -- the demo gives Carokia a high-level goal like "Patrol the north wing and report any anomalies."
   - Expected: The LLM planner decomposes this into concrete sub-steps (navigate to north wing, scan rooms, log findings, return).

5. **Demo ends** -- prints a summary of stored memories and generated plan.

## Talking Points

- Memory uses SQLite via `rusqlite` with bundled compilation -- no system SQLite dependency.
- The memory system supports both short-term (in-process) and long-term (persistent) storage.
- Planning is hybrid: rule-based planners handle known patterns while the LLM handles open-ended goals.
- Memory and planning are decoupled from the LLM -- you can use rule-only planning without any model.
