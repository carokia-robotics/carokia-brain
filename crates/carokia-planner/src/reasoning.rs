use std::sync::Arc;

use carokia_core::BrainError;
use carokia_language::{GenerateParams, LlmBackend};
use serde::{Deserialize, Serialize};

/// The type of a reasoning step in a chain-of-thought sequence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    Thought,
    Action,
    Observation,
    Answer,
}

/// A single step in a reasoning chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningStep {
    pub step_type: StepType,
    pub content: String,
}

/// A complete chain of reasoning steps with an optional final answer.
pub struct ReasoningChain {
    pub steps: Vec<ReasoningStep>,
    pub final_answer: Option<String>,
}

/// Chain-of-thought reasoner using a ReAct-style loop.
pub struct ChainOfThoughtReasoner {
    backend: Arc<dyn LlmBackend>,
    max_steps: usize,
}

impl ChainOfThoughtReasoner {
    pub fn new(backend: Arc<dyn LlmBackend>, max_steps: usize) -> Self {
        Self { backend, max_steps }
    }

    /// Run chain-of-thought reasoning on a question.
    ///
    /// Uses a ReAct loop: prompts the LLM with "Think step by step",
    /// parses Thought/Action/Observation/Answer lines from the response,
    /// and loops until an Answer is found or max_steps is reached.
    pub async fn reason(&self, question: &str) -> Result<ReasoningChain, BrainError> {
        let mut steps = Vec::new();
        let mut context = format!(
            "Answer the following question by thinking step by step.\n\
             Use the format:\n\
             Thought: <your reasoning>\n\
             Action: <action to take if needed>\n\
             Observation: <result of action>\n\
             Answer: <final answer>\n\n\
             Question: {question}\n"
        );

        for _ in 0..self.max_steps {
            let params = GenerateParams {
                max_tokens: 512,
                temperature: 0.3,
                stop_sequences: vec![],
            };

            let response = self.backend.generate(&context, &params).await?;

            let new_steps = parse_reasoning_steps(&response);
            if new_steps.is_empty() {
                // If we can't parse any steps, treat the whole response as an answer.
                let answer_step = ReasoningStep {
                    step_type: StepType::Answer,
                    content: response.trim().to_string(),
                };
                steps.push(answer_step);
                let final_answer = steps.last().map(|s| s.content.clone());
                return Ok(ReasoningChain {
                    steps,
                    final_answer,
                });
            }

            let has_answer = new_steps.iter().any(|s| s.step_type == StepType::Answer);
            for step in &new_steps {
                context.push_str(&format!(
                    "{}: {}\n",
                    step_type_label(&step.step_type),
                    step.content
                ));
            }
            steps.extend(new_steps);

            if has_answer {
                let final_answer = steps
                    .iter()
                    .rev()
                    .find(|s| s.step_type == StepType::Answer)
                    .map(|s| s.content.clone());
                return Ok(ReasoningChain {
                    steps,
                    final_answer,
                });
            }
        }

        // Max steps reached without an answer.
        Ok(ReasoningChain {
            steps,
            final_answer: None,
        })
    }
}

fn step_type_label(st: &StepType) -> &str {
    match st {
        StepType::Thought => "Thought",
        StepType::Action => "Action",
        StepType::Observation => "Observation",
        StepType::Answer => "Answer",
    }
}

/// Parse reasoning steps from an LLM response string.
fn parse_reasoning_steps(text: &str) -> Vec<ReasoningStep> {
    let mut steps = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(content) = trimmed.strip_prefix("Thought:") {
            steps.push(ReasoningStep {
                step_type: StepType::Thought,
                content: content.trim().to_string(),
            });
        } else if let Some(content) = trimmed.strip_prefix("Action:") {
            steps.push(ReasoningStep {
                step_type: StepType::Action,
                content: content.trim().to_string(),
            });
        } else if let Some(content) = trimmed.strip_prefix("Observation:") {
            steps.push(ReasoningStep {
                step_type: StepType::Observation,
                content: content.trim().to_string(),
            });
        } else if let Some(content) = trimmed.strip_prefix("Answer:") {
            steps.push(ReasoningStep {
                step_type: StepType::Answer,
                content: content.trim().to_string(),
            });
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use carokia_language::MockBackend;

    #[test]
    fn reasoning_step_serialization_roundtrip() {
        let step = ReasoningStep {
            step_type: StepType::Thought,
            content: "I need to consider the problem.".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        let restored: ReasoningStep = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.step_type, StepType::Thought);
        assert_eq!(restored.content, step.content);
    }

    #[test]
    fn step_type_serialization() {
        for st in [
            StepType::Thought,
            StepType::Action,
            StepType::Observation,
            StepType::Answer,
        ] {
            let json = serde_json::to_string(&st).unwrap();
            let restored: StepType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, st);
        }
    }

    #[test]
    fn parse_reasoning_steps_basic() {
        let text = "Thought: I need to break this down.\n\
                    Action: Look up the formula.\n\
                    Observation: The formula is E=mc^2.\n\
                    Answer: The answer is 42.";
        let steps = parse_reasoning_steps(text);
        assert_eq!(steps.len(), 4);
        assert_eq!(steps[0].step_type, StepType::Thought);
        assert_eq!(steps[1].step_type, StepType::Action);
        assert_eq!(steps[2].step_type, StepType::Observation);
        assert_eq!(steps[3].step_type, StepType::Answer);
        assert_eq!(steps[3].content, "The answer is 42.");
    }

    #[test]
    fn parse_reasoning_steps_empty() {
        let steps = parse_reasoning_steps("Just some random text.");
        assert!(steps.is_empty());
    }

    #[tokio::test]
    async fn reasoner_with_mock_produces_chain() {
        let mock_response = "Thought: Let me think about this.\n\
                            Observation: I know the answer.\n\
                            Answer: 42";
        let backend = Arc::new(MockBackend::new(mock_response));
        let reasoner = ChainOfThoughtReasoner::new(backend, 5);
        let chain = reasoner
            .reason("What is the meaning of life?")
            .await
            .unwrap();
        assert!(!chain.steps.is_empty());
        assert!(chain.final_answer.is_some());
        assert_eq!(chain.final_answer.unwrap(), "42");
    }

    #[tokio::test]
    async fn reasoner_terminates_at_max_steps() {
        // Mock that never produces an Answer
        let mock_response = "Thought: Still thinking...";
        let backend = Arc::new(MockBackend::new(mock_response));
        let reasoner = ChainOfThoughtReasoner::new(backend, 3);
        let chain = reasoner.reason("Impossible question").await.unwrap();
        // Should have at most 3 * (steps per iteration) steps
        assert!(chain.steps.len() <= 3);
        assert!(chain.final_answer.is_none());
    }

    #[tokio::test]
    async fn reasoner_handles_unparseable_response() {
        let mock_response = "I don't know what format you want.";
        let backend = Arc::new(MockBackend::new(mock_response));
        let reasoner = ChainOfThoughtReasoner::new(backend, 5);
        let chain = reasoner.reason("Test question").await.unwrap();
        // Should treat it as an Answer
        assert!(chain.final_answer.is_some());
    }

    #[tokio::test]
    async fn reasoner_multi_step_thought_then_answer() {
        let mock_response = "Thought: Step one - understand the question.\n\
                            Thought: Step two - the capital of France is Paris.\n\
                            Answer: Paris";
        let backend = Arc::new(MockBackend::new(mock_response));
        let reasoner = ChainOfThoughtReasoner::new(backend, 5);
        let chain = reasoner
            .reason("What is the capital of France?")
            .await
            .unwrap();
        assert_eq!(chain.steps.len(), 3);
        assert_eq!(chain.final_answer.unwrap(), "Paris");
    }
}
