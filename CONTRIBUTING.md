# Contributing to AI Agent Framework

Thank you for your interest in contributing to the AI Agent Framework! This document provides guidelines for extending the framework with new capabilities.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style Guidelines](#code-style-guidelines)
- [Running Tests](#running-tests)
- [Extending the Framework](#extending-the-framework)
  - [Adding New LLM Providers](#adding-new-llm-providers)
  - [Creating Custom Tools](#creating-custom-tools)
  - [Implementing Custom Guardrails](#implementing-custom-guardrails)
  - [Creating Custom Rules](#creating-custom-rules)
- [Submitting Changes](#submitting-changes)

## Getting Started

The AI Agent Framework is built as a Cargo workspace with 12 member crates organized into three architectural layers:

1. **Foundation Layer**: `core`, `config`, `communication`
2. **Capability Layer**: `llm`, `memory`, `tools`
3. **Intelligence Layer**: `planner`, `executor`, `guardrails`, `rules`, `cli`, `examples`

Before contributing, familiarize yourself with the project structure by reading the [README.md](README.md) and exploring the codebase.

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)
- API keys for testing (optional): `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd ai-agent-framework

# Build all crates
cargo build --workspace

# Build in release mode
cargo build --release --workspace

# Build examples
cargo build --examples
```

### Running Examples

```bash
# Set your API key
export OPENAI_API_KEY="your-key-here"

# Run an example
cargo run --example chatbot
cargo run --example research
cargo run --example file_manager
```

## Code Style Guidelines

### General Principles

- **Clarity over cleverness**: Write code that is easy to understand and maintain
- **Documentation first**: All public APIs must have rustdoc comments with examples
- **Error handling**: Use `Result` types and provide descriptive error messages
- **Type safety**: Leverage Rust's type system to prevent errors at compile time

### Formatting

We use `rustfmt` for consistent code formatting:

```bash
# Format all code
cargo fmt --all

# Check formatting without modifying files
cargo fmt --all -- --check
```

### Linting

We use `clippy` for additional code quality checks:

```bash
# Run clippy on all crates
cargo clippy --workspace -- -D warnings
```

All code must pass clippy without warnings before submission.

### Documentation

- **Public APIs**: Must have rustdoc comments explaining purpose, parameters, return values, and errors
- **Examples**: Include usage examples in doc comments where helpful
- **Module docs**: Each module should have a module-level doc comment explaining its purpose

Example:

```rust
/// Executes the tool with the given parameters.
/// 
/// # Arguments
/// * `params` - JSON value containing the tool parameters
/// 
/// # Returns
/// A JSON value containing the tool's result
/// 
/// # Errors
/// Returns an error if parameters are invalid or execution fails
/// 
/// # Example
/// ```rust,ignore
/// let params = json!({"operation": "add", "a": 5, "b": 3});
/// let result = calculator.execute(params).await?;
/// ```
async fn execute(&self, params: Value) -> Result<Value>;
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `LLMProvider`, `ToolRegistry`)
- **Functions/methods**: `snake_case` (e.g., `send_message`, `create_provider`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Modules**: `snake_case` (e.g., `file_reader`, `rate_limit`)

## Running Tests

### Unit Tests

Run tests for all crates:

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p core
cargo test -p tools
```

### Integration Tests

Integration tests are located in the `tests/` directory:

```bash
# Run all integration tests
cargo test --test agent_flow
cargo test --test tool_execution
cargo test --test guardrails_integration

# Run integration tests that require API keys (marked with #[ignore])
cargo test --test openai_integration -- --ignored
cargo test --test anthropic_integration -- --ignored
```

### Test Coverage

When adding new functionality:

1. Write unit tests for core logic in `#[cfg(test)]` modules
2. Add integration tests for cross-crate interactions
3. Ensure error cases are tested
4. Test edge cases and boundary conditions

## Extending the Framework

### Adding New LLM Providers

To add support for a new LLM provider (e.g., Google PaLM, Cohere):

#### 1. Create Provider Module

Create a new module in `llm/src/`:

```bash
mkdir llm/src/newprovider
touch llm/src/newprovider/mod.rs
touch llm/src/newprovider/types.rs
```

#### 2. Define API Types

In `llm/src/newprovider/types.rs`, define request and response types:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct NewProviderRequest {
    pub model: String,
    pub messages: Vec<NewProviderMessage>,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct NewProviderMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct NewProviderResponse {
    pub text: String,
    // Add other fields as needed
}
```

#### 3. Implement the Provider

In `llm/src/newprovider/mod.rs`:

```rust
pub mod types;

use agent_core::{AgentError, Message, Result, Role};
use async_trait::async_trait;
use communication::ApiClient;
use config::LLMConfig;

use crate::LLMProvider;
pub use types::{NewProviderRequest, NewProviderResponse, NewProviderMessage};

pub struct NewProvider {
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: usize,
    client: ApiClient,
}

impl NewProvider {
    pub fn new(config: &LLMConfig) -> Result<Self> {
        Ok(Self {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            client: ApiClient::new(),
        })
    }

    fn convert_message(message: &Message) -> NewProviderMessage {
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };

        NewProviderMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }
}

#[async_trait]
impl LLMProvider for NewProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        let provider_messages: Vec<_> = messages
            .iter()
            .map(Self::convert_message)
            .collect();

        let request = NewProviderRequest {
            model: self.model.clone(),
            messages: provider_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let url = "https://api.newprovider.com/v1/completions";
        
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(self.client.timeout())
            .send()
            .await
            .map_err(|e| {
                AgentError::LLMProvider(format!("NewProvider API error: {}", e))
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AgentError::LLMProvider(format!(
                "NewProvider API HTTP error: {}", error_text
            )));
        }

        let completion: NewProviderResponse = response.json().await
            .map_err(|e| AgentError::LLMProvider(format!(
                "Failed to parse response: {}", e
            )))?;

        Ok(completion.text)
    }
}
```

#### 4. Update Factory

Add your provider to `llm/src/factory.rs`:

```rust
use crate::newprovider::NewProvider;

pub fn create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(config)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(config)?)),
        "newprovider" => Ok(Box::new(NewProvider::new(config)?)),
        _ => Err(AgentError::Config(format!("Unknown provider: {}", config.provider))),
    }
}
```

#### 5. Update Exports

Add to `llm/src/lib.rs`:

```rust
pub mod newprovider;
pub use newprovider::NewProvider;
```

#### 6. Add Tests

Create integration test in `tests/newprovider_integration.rs`:

```rust
#[cfg(test)]
mod tests {
    use agent_core::{Message, Role};
    use config::LLMConfig;
    use llm::{LLMProvider, NewProvider};

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_newprovider_send_message() {
        let config = LLMConfig {
            provider: "newprovider".to_string(),
            model: "model-name".to_string(),
            api_key: std::env::var("NEWPROVIDER_API_KEY")
                .expect("NEWPROVIDER_API_KEY not set"),
            temperature: 0.7,
            max_tokens: 100,
        };

        let provider = NewProvider::new(&config).unwrap();
        let messages = vec![Message::user("Hello!")];
        
        let response = provider.send_message(&messages).await;
        assert!(response.is_ok());
    }
}
```

### Creating Custom Tools

Tools extend agent capabilities with external actions. Here's how to create a custom tool:

#### 1. Create Tool Module

Create a new file in `tools/src/` (e.g., `tools/src/weather.rs`):

```rust
use agent_core::{AgentError, Result};
use async_trait::async_trait;
use serde_json::{json, Value};

use crate::Tool;

/// Tool for fetching weather information
pub struct WeatherTool {
    api_key: String,
}

impl WeatherTool {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Fetches current weather information for a given location"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name or location"
                }
            },
            "required": ["location"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        // Extract and validate parameters
        let location = params
            .get("location")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: "Missing or invalid 'location' parameter".to_string(),
            })?;

        // Call weather API (example)
        let client = reqwest::Client::new();
        let url = format!(
            "https://api.weatherapi.com/v1/current.json?key={}&q={}",
            self.api_key, location
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: format!("API request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: format!("API returned status: {}", response.status()),
            });
        }

        let weather_data: Value = response.json().await.map_err(|e| {
            AgentError::ToolExecution {
                tool_name: self.name().to_string(),
                reason: format!("Failed to parse response: {}", e),
            }
        })?;

        // Return formatted result
        Ok(json!({
            "location": location,
            "temperature": weather_data["current"]["temp_c"],
            "condition": weather_data["current"]["condition"]["text"],
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weather_tool_parameters() {
        let tool = WeatherTool::new("test_key".to_string());
        
        assert_eq!(tool.name(), "weather");
        assert!(!tool.description().is_empty());
        
        let schema = tool.parameters_schema();
        assert!(schema["properties"]["location"].is_object());
    }

    #[tokio::test]
    async fn test_weather_tool_missing_parameter() {
        let tool = WeatherTool::new("test_key".to_string());
        let params = json!({});
        
        let result = tool.execute(params).await;
        assert!(result.is_err());
    }
}
```

#### 2. Register Tool

Add to `tools/src/lib.rs`:

```rust
pub mod weather;
pub use weather::WeatherTool;
```

#### 3. Use in Agent

Register the tool when initializing your agent:

```rust
use tools::{ToolRegistry, WeatherTool};

let mut registry = ToolRegistry::new();
registry.register(Box::new(WeatherTool::new(api_key)));
```

### Implementing Custom Guardrails

Guardrails validate plans before execution to enforce safety constraints:

#### 1. Create Guardrail Module

Create a new file in `guardrails/src/` (e.g., `guardrails/src/api_whitelist.rs`):

```rust
use agent_core::{AgentError, Result};
use planner::{Plan, Step};

use crate::Guardrail;

/// Guardrail that restricts API calls to a whitelist of domains
pub struct ApiWhitelistGuardrail {
    allowed_domains: Vec<String>,
}

impl ApiWhitelistGuardrail {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self { allowed_domains }
    }

    fn is_domain_allowed(&self, url: &str) -> bool {
        self.allowed_domains.iter().any(|domain| url.contains(domain))
    }
}

impl Guardrail for ApiWhitelistGuardrail {
    fn name(&self) -> &str {
        "api_whitelist"
    }

    fn validate(&self, plan: &Plan) -> Result<()> {
        for step in &plan.steps {
            if let Step::ToolCall(tool_call) = step {
                // Check if this is a web-related tool
                if tool_call.tool_name == "web_search" || tool_call.tool_name == "http_request" {
                    // Extract URL from parameters
                    if let Some(url) = tool_call.parameters.get("url")
                        .and_then(|v| v.as_str())
                    {
                        if !self.is_domain_allowed(url) {
                            return Err(AgentError::GuardrailViolation(format!(
                                "API call to '{}' is not allowed. Permitted domains: {:?}",
                                url, self.allowed_domains
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use planner::ToolCall;
    use serde_json::json;

    #[test]
    fn test_allowed_domain() {
        let guardrail = ApiWhitelistGuardrail::new(vec![
            "api.example.com".to_string(),
        ]);

        let plan = Plan {
            steps: vec![Step::ToolCall(ToolCall {
                tool_name: "web_search".to_string(),
                parameters: json!({"url": "https://api.example.com/search"}),
            })],
            reasoning: "Test".to_string(),
        };

        assert!(guardrail.validate(&plan).is_ok());
    }

    #[test]
    fn test_blocked_domain() {
        let guardrail = ApiWhitelistGuardrail::new(vec![
            "api.example.com".to_string(),
        ]);

        let plan = Plan {
            steps: vec![Step::ToolCall(ToolCall {
                tool_name: "web_search".to_string(),
                parameters: json!({"url": "https://malicious.com/data"}),
            })],
            reasoning: "Test".to_string(),
        };

        assert!(guardrail.validate(&plan).is_err());
    }
}
```

#### 2. Register Guardrail

Add to `guardrails/src/lib.rs`:

```rust
pub mod api_whitelist;
pub use api_whitelist::ApiWhitelistGuardrail;
```

#### 3. Use in Agent

Register the guardrail when initializing your agent:

```rust
use guardrails::{GuardrailRegistry, ApiWhitelistGuardrail};

let mut registry = GuardrailRegistry::new();
registry.register(Box::new(ApiWhitelistGuardrail::new(vec![
    "api.openai.com".to_string(),
    "api.anthropic.com".to_string(),
])));
```

### Creating Custom Rules

Rules customize agent behavior by modifying the planning context:

#### 1. Create Rule Module

Create a new file in `rules/src/` (e.g., `rules/src/language.rs`):

```rust
use crate::{PlanningContext, Rule};

/// Rule that sets the response language
pub struct LanguageRule {
    language: String,
}

impl LanguageRule {
    pub fn new(language: String) -> Self {
        Self { language }
    }
}

impl Rule for LanguageRule {
    fn name(&self) -> &str {
        "language"
    }

    fn priority(&self) -> u32 {
        75 // Applied after tone (50) but before length (100)
    }

    fn apply(&self, context: &mut PlanningContext) {
        // Modify system prompt to include language instruction
        let language_instruction = format!(
            "\n\nIMPORTANT: Respond in {}. All responses must be in this language.",
            self.language
        );
        context.system_prompt.push_str(&language_instruction);

        // Add metadata
        context.metadata.insert(
            "language".to_string(),
            self.language.clone(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_rule() {
        let rule = LanguageRule::new("Spanish".to_string());
        let mut context = PlanningContext::new("Test prompt".to_string());

        rule.apply(&mut context);

        assert!(context.system_prompt.contains("Spanish"));
        assert_eq!(context.metadata.get("language"), Some(&"Spanish".to_string()));
    }
}
```

#### 2. Register Rule

Add to `rules/src/lib.rs`:

```rust
pub mod language;
pub use language::LanguageRule;
```

#### 3. Use in Agent

Register the rule when initializing your agent:

```rust
use rules::{RuleEngine, LanguageRule};

let mut engine = RuleEngine::new();
engine.add_rule(Box::new(LanguageRule::new("French".to_string())));
```

## Submitting Changes

### Before Submitting

1. **Run all tests**: `cargo test --workspace`
2. **Check formatting**: `cargo fmt --all -- --check`
3. **Run clippy**: `cargo clippy --workspace -- -D warnings`
4. **Update documentation**: Ensure all public APIs have rustdoc comments
5. **Add examples**: Include usage examples for new features

### Pull Request Guidelines

- **Clear description**: Explain what your changes do and why
- **Reference issues**: Link to related issues or feature requests
- **Small, focused changes**: Keep PRs focused on a single feature or fix
- **Tests included**: All new functionality must have tests
- **Documentation updated**: Update README.md and relevant docs

### Commit Message Format

Use clear, descriptive commit messages:

```
Add WeatherTool for fetching weather data

- Implement Tool trait for weather API integration
- Add parameter validation and error handling
- Include unit tests for core functionality
```

## Questions?

If you have questions about contributing, please:

1. Check the [README.md](README.md) for project overview
2. Review existing code for patterns and examples
3. Open an issue for discussion before starting major changes

Thank you for contributing to the AI Agent Framework!
