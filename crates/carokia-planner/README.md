# carokia-planner

Goal decomposition and task planning for Carokia. Provides a `RulePlanner` that breaks goals into concrete tasks using pattern-matching rules, and an optional `LlmPlanner` (behind the `llm` feature) that uses a language model for open-ended goal decomposition. Also includes a chain-of-thought reasoner (behind the `reasoning` feature) for multi-step problem solving.
