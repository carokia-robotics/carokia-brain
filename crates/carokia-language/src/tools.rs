use async_trait::async_trait;
use carokia_core::BrainError;

/// A tool that can be called programmatically by the conversation system.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, input: &str) -> Result<String, BrainError>;
}

/// Registry of available tools.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Format all tool descriptions for inclusion in a system prompt.
    pub fn list_descriptions(&self) -> String {
        self.tools
            .iter()
            .map(|t| format!("- {}: {}", t.name(), t.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Find a tool by name.
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    /// Execute a tool by name with the given input.
    pub async fn execute(&self, name: &str, input: &str) -> Result<String, BrainError> {
        let tool = self
            .find(name)
            .ok_or_else(|| BrainError::Language(format!("Tool not found: {name}")))?;
        tool.execute(input).await
    }
}

/// Returns the current date and time.
pub struct CurrentTimeTool;

#[async_trait]
impl Tool for CurrentTimeTool {
    fn name(&self) -> &str {
        "current_time"
    }

    fn description(&self) -> &str {
        "Returns the current date and time"
    }

    async fn execute(&self, _input: &str) -> Result<String, BrainError> {
        let now = chrono::Local::now();
        Ok(now.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}

/// Evaluates simple math expressions (basic arithmetic).
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluates simple arithmetic expressions (e.g. '2+2', '10*3-5')"
    }

    async fn execute(&self, input: &str) -> Result<String, BrainError> {
        eval_math(input).map_err(|e| BrainError::Language(format!("Calculator error: {e}")))
    }
}

/// Simple recursive-descent math evaluator supporting +, -, *, /, parentheses.
fn eval_math(expr: &str) -> Result<String, String> {
    let tokens = tokenize(expr)?;
    let mut pos = 0;
    let result = parse_expr(&tokens, &mut pos)?;
    if pos != tokens.len() {
        return Err(format!("Unexpected token at position {pos}"));
    }
    // Format nicely: if integer result, no decimal point
    if result == result.floor() && result.abs() < 1e15 {
        Ok(format!("{}", result as i64))
    } else {
        Ok(format!("{result}"))
    }
}

#[derive(Debug, Clone)]
enum MathToken {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

fn tokenize(expr: &str) -> Result<Vec<MathToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' => i += 1,
            '+' => {
                tokens.push(MathToken::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(MathToken::Minus);
                i += 1;
            }
            '*' => {
                tokens.push(MathToken::Star);
                i += 1;
            }
            '/' => {
                tokens.push(MathToken::Slash);
                i += 1;
            }
            '(' => {
                tokens.push(MathToken::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(MathToken::RParen);
                i += 1;
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num: f64 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid number: {num_str}"))?;
                tokens.push(MathToken::Number(num));
            }
            c => return Err(format!("Unexpected character: {c}")),
        }
    }
    Ok(tokens)
}

fn parse_expr(tokens: &[MathToken], pos: &mut usize) -> Result<f64, String> {
    let mut left = parse_term(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            MathToken::Plus => {
                *pos += 1;
                left += parse_term(tokens, pos)?;
            }
            MathToken::Minus => {
                *pos += 1;
                left -= parse_term(tokens, pos)?;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_term(tokens: &[MathToken], pos: &mut usize) -> Result<f64, String> {
    let mut left = parse_unary(tokens, pos)?;
    while *pos < tokens.len() {
        match tokens[*pos] {
            MathToken::Star => {
                *pos += 1;
                left *= parse_unary(tokens, pos)?;
            }
            MathToken::Slash => {
                *pos += 1;
                let right = parse_unary(tokens, pos)?;
                if right == 0.0 {
                    return Err("Division by zero".to_string());
                }
                left /= right;
            }
            _ => break,
        }
    }
    Ok(left)
}

fn parse_unary(tokens: &[MathToken], pos: &mut usize) -> Result<f64, String> {
    if *pos < tokens.len() {
        if let MathToken::Minus = tokens[*pos] {
            *pos += 1;
            let val = parse_atom(tokens, pos)?;
            return Ok(-val);
        }
    }
    parse_atom(tokens, pos)
}

fn parse_atom(tokens: &[MathToken], pos: &mut usize) -> Result<f64, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of expression".to_string());
    }
    match tokens[*pos] {
        MathToken::Number(n) => {
            *pos += 1;
            Ok(n)
        }
        MathToken::LParen => {
            *pos += 1;
            let val = parse_expr(tokens, pos)?;
            if *pos >= tokens.len() {
                return Err("Missing closing parenthesis".to_string());
            }
            if let MathToken::RParen = tokens[*pos] {
                *pos += 1;
                Ok(val)
            } else {
                Err("Expected closing parenthesis".to_string())
            }
        }
        _ => Err(format!("Unexpected token: {:?}", tokens[*pos])),
    }
}

/// Runs allowlisted shell commands.
pub struct ShellCommandTool;

const ALLOWED_COMMANDS: &[&str] = &["ls", "pwd", "date", "uptime", "whoami"];

#[async_trait]
impl Tool for ShellCommandTool {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        "Runs allowlisted shell commands: ls, pwd, date, uptime, whoami"
    }

    async fn execute(&self, input: &str) -> Result<String, BrainError> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts
            .first()
            .ok_or_else(|| BrainError::Language("Empty command".into()))?;

        if !ALLOWED_COMMANDS.contains(cmd) {
            return Err(BrainError::Language(format!(
                "Command '{cmd}' is not allowed. Allowed: {}",
                ALLOWED_COMMANDS.join(", ")
            )));
        }

        let output = tokio::process::Command::new(cmd)
            .args(&parts[1..])
            .output()
            .await
            .map_err(|e| BrainError::Language(format!("Shell error: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(stdout.to_string())
        } else {
            Ok(format!("Error: {stderr}"))
        }
    }
}

/// Searches stored memories by keyword (simple substring search).
pub struct MemorySearchTool {
    memories: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
}

impl MemorySearchTool {
    pub fn new(memories: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>) -> Self {
        Self { memories }
    }
}

#[async_trait]
impl Tool for MemorySearchTool {
    fn name(&self) -> &str {
        "memory_search"
    }

    fn description(&self) -> &str {
        "Searches stored memories for a given keyword"
    }

    async fn execute(&self, input: &str) -> Result<String, BrainError> {
        let memories = self.memories.lock().await;
        let query = input.to_lowercase();
        let matches: Vec<&String> = memories
            .iter()
            .filter(|m| m.to_lowercase().contains(&query))
            .collect();

        if matches.is_empty() {
            Ok("No matching memories found.".to_string())
        } else {
            Ok(matches
                .iter()
                .map(|m| format!("- {m}"))
                .collect::<Vec<_>>()
                .join("\n"))
        }
    }
}

/// Create a ToolRegistry with built-in tools.
pub fn default_tools() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(CurrentTimeTool));
    registry.register(Box::new(CalculatorTool));
    registry.register(Box::new(ShellCommandTool));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tool_registry_finds_tool() {
        let registry = default_tools();
        assert!(registry.find("current_time").is_some());
        assert!(registry.find("calculator").is_some());
        assert!(registry.find("shell").is_some());
        assert!(registry.find("nonexistent").is_none());
    }

    #[tokio::test]
    async fn tool_registry_executes_tool() {
        let registry = default_tools();
        let result = registry.execute("calculator", "2+2").await.unwrap();
        assert_eq!(result, "4");
    }

    #[tokio::test]
    async fn tool_registry_execute_not_found() {
        let registry = default_tools();
        let result = registry.execute("nonexistent", "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn current_time_tool_returns_valid_date() {
        let tool = CurrentTimeTool;
        let result = tool.execute("").await.unwrap();
        // Should contain a year
        assert!(result.contains("20"));
        // Should parse as a date
        assert!(chrono::NaiveDateTime::parse_from_str(&result, "%Y-%m-%d %H:%M:%S").is_ok());
    }

    #[tokio::test]
    async fn calculator_evaluates_basic() {
        let tool = CalculatorTool;
        assert_eq!(tool.execute("2+2").await.unwrap(), "4");
        assert_eq!(tool.execute("10*3-5").await.unwrap(), "25");
        assert_eq!(tool.execute("(1+2)*3").await.unwrap(), "9");
        assert_eq!(tool.execute("100/4").await.unwrap(), "25");
    }

    #[tokio::test]
    async fn calculator_handles_negation() {
        let tool = CalculatorTool;
        assert_eq!(tool.execute("-5+3").await.unwrap(), "-2");
    }

    #[tokio::test]
    async fn shell_tool_rejects_disallowed() {
        let tool = ShellCommandTool;
        let result = tool.execute("rm -rf /").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn shell_tool_runs_allowed() {
        let tool = ShellCommandTool;
        let result = tool.execute("whoami").await.unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn memory_search_tool_finds_matches() {
        let memories = std::sync::Arc::new(tokio::sync::Mutex::new(vec![
            "The sky is blue".to_string(),
            "Cats are fluffy".to_string(),
            "The sky at night is dark".to_string(),
        ]));
        let tool = MemorySearchTool::new(memories);
        let result = tool.execute("sky").await.unwrap();
        assert!(result.contains("sky is blue"));
        assert!(result.contains("sky at night"));
        assert!(!result.contains("fluffy"));
    }

    #[tokio::test]
    async fn memory_search_tool_no_matches() {
        let memories =
            std::sync::Arc::new(tokio::sync::Mutex::new(vec!["hello world".to_string()]));
        let tool = MemorySearchTool::new(memories);
        let result = tool.execute("xyz").await.unwrap();
        assert_eq!(result, "No matching memories found.");
    }

    #[test]
    fn list_descriptions_formats_tools() {
        let registry = default_tools();
        let desc = registry.list_descriptions();
        assert!(desc.contains("current_time"));
        assert!(desc.contains("calculator"));
        assert!(desc.contains("shell"));
    }
}
