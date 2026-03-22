use std::sync::Arc;

use async_trait::async_trait;
use carokia_core::BrainError;
use carokia_language::{GenerateParams, LlmBackend};

use crate::{Goal, Planner, TaskNode};

/// LLM-powered planner that decomposes goals into subtasks using an LLM backend.
pub struct LlmPlanner {
    backend: Arc<dyn LlmBackend>,
}

impl LlmPlanner {
    pub fn new(backend: Arc<dyn LlmBackend>) -> Self {
        Self { backend }
    }
}

/// Try to extract a JSON object from text that may contain surrounding prose.
fn extract_json(text: &str) -> Option<&str> {
    // Try to find JSON between curly braces
    let start = text.find('{')?;
    let mut depth = 0;
    let bytes = text.as_bytes();
    for i in start..bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse the LLM response into TaskNode entries.
fn parse_subtasks(goal: &Goal, response: &str) -> Result<Vec<TaskNode>, BrainError> {
    let json_str = extract_json(response).ok_or_else(|| {
        BrainError::Planner("No JSON found in LLM response".into())
    })?;

    let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        BrainError::Planner(format!("JSON parse error: {e}"))
    })?;

    let subtasks = parsed["subtasks"]
        .as_array()
        .ok_or_else(|| BrainError::Planner("No 'subtasks' array in response".into()))?;

    let mut nodes: Vec<TaskNode> = Vec::new();
    for task in subtasks {
        let description = task["description"]
            .as_str()
            .unwrap_or("Unnamed task")
            .to_string();

        let mut node = TaskNode::new(&goal.id, description);

        // Parse depends_on if present
        if let Some(deps) = task["depends_on"].as_array() {
            for dep in deps {
                if let Some(dep_idx) = dep.as_u64() {
                    // Reference by index into already-created nodes
                    if let Some(dep_node) = nodes.get(dep_idx as usize) {
                        node.depends_on.push(dep_node.id.clone());
                    }
                }
            }
        }

        nodes.push(node);
    }

    if nodes.is_empty() {
        return Err(BrainError::Planner("LLM returned empty subtask list".into()));
    }

    Ok(nodes)
}

#[async_trait]
impl Planner for LlmPlanner {
    async fn decompose(&self, goal: &Goal) -> Result<Vec<TaskNode>, BrainError> {
        let prompt = format!(
            "Decompose this goal into 3-7 ordered subtasks. Return JSON only.\n\
            Goal: {}\n\
            Format: {{\"subtasks\": [{{\"description\": \"...\", \"depends_on\": []}}]}}\n\
            JSON:",
            goal.description
        );

        let response = self
            .backend
            .generate(
                &prompt,
                &GenerateParams {
                    max_tokens: 512,
                    temperature: 0.3,
                    ..Default::default()
                },
            )
            .await?;

        // Try to parse LLM output; fall back to single task on failure
        match parse_subtasks(goal, &response) {
            Ok(nodes) => Ok(nodes),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to parse LLM plan, falling back to single task"
                );
                Ok(vec![TaskNode::new(&goal.id, &goal.description)])
            }
        }
    }

    async fn replan(
        &self,
        goal: &Goal,
        failed_tasks: &[TaskNode],
    ) -> Result<Vec<TaskNode>, BrainError> {
        let failed_descriptions: Vec<&str> =
            failed_tasks.iter().map(|t| t.description.as_str()).collect();

        let prompt = format!(
            "The following subtasks failed for a goal. Create a revised plan with 3-7 subtasks. Return JSON only.\n\
            Goal: {}\n\
            Failed tasks: {}\n\
            Format: {{\"subtasks\": [{{\"description\": \"...\", \"depends_on\": []}}]}}\n\
            JSON:",
            goal.description,
            failed_descriptions.join(", ")
        );

        let response = self
            .backend
            .generate(
                &prompt,
                &GenerateParams {
                    max_tokens: 512,
                    temperature: 0.3,
                    ..Default::default()
                },
            )
            .await?;

        match parse_subtasks(goal, &response) {
            Ok(nodes) => Ok(nodes),
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Failed to parse LLM replan, falling back to single task"
                );
                Ok(vec![TaskNode::new(&goal.id, &goal.description)])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_language::MockBackend;

    fn mock_json_response() -> String {
        r#"{"subtasks": [
            {"description": "Gather ingredients", "depends_on": []},
            {"description": "Mix batter", "depends_on": [0]},
            {"description": "Bake in oven", "depends_on": [1]},
            {"description": "Let cool", "depends_on": [2]},
            {"description": "Decorate", "depends_on": [3]}
        ]}"#
        .to_string()
    }

    fn mock_json_with_prose() -> String {
        "Sure, here is the plan:\n\n".to_string()
            + &mock_json_response()
            + "\n\nI hope this helps!"
    }

    #[tokio::test]
    async fn llm_planner_decompose() {
        let backend = Arc::new(MockBackend::new(mock_json_response()));
        let planner = LlmPlanner::new(backend);
        let goal = Goal::new("Bake a cake", 5);
        let tasks = planner.decompose(&goal).await.unwrap();

        assert_eq!(tasks.len(), 5);
        assert_eq!(tasks[0].description, "Gather ingredients");
        assert_eq!(tasks[4].description, "Decorate");
        // Check dependency chain
        assert!(tasks[1].depends_on.contains(&tasks[0].id));
    }

    #[tokio::test]
    async fn llm_planner_handles_prose_around_json() {
        let backend = Arc::new(MockBackend::new(mock_json_with_prose()));
        let planner = LlmPlanner::new(backend);
        let goal = Goal::new("Bake a cake", 5);
        let tasks = planner.decompose(&goal).await.unwrap();
        assert_eq!(tasks.len(), 5);
    }

    #[tokio::test]
    async fn llm_planner_fallback_on_bad_json() {
        let backend = Arc::new(MockBackend::new("This is not JSON at all."));
        let planner = LlmPlanner::new(backend);
        let goal = Goal::new("Do something", 3);
        let tasks = planner.decompose(&goal).await.unwrap();
        // Should fall back to single task
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].description, "Do something");
    }

    #[tokio::test]
    async fn llm_planner_replan() {
        let backend = Arc::new(MockBackend::new(mock_json_response()));
        let planner = LlmPlanner::new(backend);
        let goal = Goal::new("Bake a cake", 5);
        let failed = vec![TaskNode::new(&goal.id, "Failed task")];
        let tasks = planner.replan(&goal, &failed).await.unwrap();
        assert!(!tasks.is_empty());
    }

    #[test]
    fn extract_json_from_prose() {
        let text = "Here is the answer: {\"key\": \"value\"} that's it.";
        let json = extract_json(text).unwrap();
        assert_eq!(json, "{\"key\": \"value\"}");
    }

    #[test]
    fn extract_json_nested() {
        let text = r#"{"outer": {"inner": 1}}"#;
        let json = extract_json(text).unwrap();
        assert_eq!(json, text);
    }

    #[test]
    fn extract_json_none_when_missing() {
        let text = "No JSON here!";
        assert!(extract_json(text).is_none());
    }
}
