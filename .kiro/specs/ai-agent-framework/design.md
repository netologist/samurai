# Design Document

## Overview

This design document provides a comprehensive architecture for an educational AI agent framework in Rust, structured as a daily implementation plan suitable for a blog series. The framework follows a modular workspace design where each crate has clear responsibilities and minimal coupling. The design emphasizes incremental development, hands-on learning, and production-quality patterns.

The framework architecture consists of three layers:
1. **Foundation Layer**: core, config, communication (Days 1-3)
2. **Capability Layer**: llm, memory, tools (Days 4-7)
3. **Intelligence Layer**: planner, executor, guardrails, rules, cli, examples (Days 8-14)

## Conceptual Checklist

This high-level checklist covers the major phases of the project:

1. **Project Setup & Core Abstractions** - Establish workspace structure, define fundamental traits, and configure error handling
2. **LLM Integration & Configuration** - Implement provider interfaces, configuration management, and API communication
3. **Agent Capabilities** - Build memory systems, tool registry, and execution primitives
4. **Planning & Orchestration** - Create the planner and executor that coordinate agent behavior
5. **Safety & Customization** - Implement guardrails, rules engine, and validation systems
6. **User Interface & Examples** - Build CLI, create example agents, and write documentation
7. **Testing & Polish** - Add integration tests, refine error handling, and prepare for publication


## Architecture

### Workspace Structure

The project uses a Cargo workspace to organize multiple related crates. This provides:
- Shared dependency versions across all crates
- Unified build and test commands
- Clear module boundaries that prevent tight coupling
- Easy navigation for learners following the blog series

```
ai-agent-framework/
├── Cargo.toml              # Workspace manifest
├── README.md               # Project overview
├── core/                   # Fundamental traits and types
├── config/                 # Configuration management
├── communication/          # HTTP client and API utilities
├── llm/                    # LLM provider implementations
├── memory/                 # Conversation storage
├── tools/                  # Tool trait and implementations
├── planner/                # Task decomposition
├── executor/               # Step execution
├── guardrails/             # Safety validation
├── rules/                  # Behavior customization
├── cli/                    # Command-line interface
└── examples/               # Example agents
```

### Component Dependencies

```
cli → planner, executor, guardrails, rules, memory, llm, tools, config, core
examples → planner, executor, guardrails, rules, memory, llm, tools, config, core
executor → tools, memory, llm, core
planner → tools, llm, memory, core
guardrails → tools, core
rules → core
tools → core
memory → core
llm → communication, config, core
communication → core
config → core
core → (no internal dependencies)
```

This dependency graph ensures:
- The core crate has no dependencies on other framework crates
- Lower-level crates (llm, memory, tools) don't depend on higher-level orchestration
- The cli and examples are leaf nodes that compose all capabilities


## Components and Interfaces

### Core Crate

**Purpose**: Defines fundamental traits, types, and errors used throughout the framework.

**Key Types**:
- `Message`: Represents a conversation message with role (System, User, Assistant), content, and timestamp
- `AgentError`: Common error type using thiserror for all framework errors
- `Result<T>`: Type alias for `std::result::Result<T, AgentError>`

**Key Traits**:
- None initially - traits will be defined in their respective domain crates

**Dependencies**: 
- `serde` (1.0) - Serialization for messages and configuration
- `thiserror` (1.0) - Ergonomic error definitions
- `chrono` (0.4) - Timestamp handling

### Config Crate

**Purpose**: Manages framework configuration including LLM settings, API keys, and agent parameters.

**Key Types**:
- `AgentConfig`: Top-level configuration structure
- `LLMConfig`: Provider-specific settings (provider type, model, temperature, max_tokens)
- `MemoryConfig`: Memory system settings (max_messages, token_budget)
- `ToolConfig`: Tool enablement and parameters

**Key Functions**:
- `load_from_file(path: &Path) -> Result<AgentConfig>`: Parse YAML/TOML config
- `from_env() -> Result<AgentConfig>`: Build config from environment variables
- `merge(file_config, env_config) -> AgentConfig`: Combine file and env configs

**Dependencies**:
- `serde` (1.0) - Deserialization
- `serde_yaml` (0.9) - YAML parsing
- `toml` (0.8) - TOML parsing (alternative)
- `core` - Error types


### Communication Crate

**Purpose**: Provides HTTP client utilities and API communication primitives.

**Key Types**:
- `ApiClient`: Wrapper around reqwest with retry logic and error handling
- `ApiRequest`: Builder for HTTP requests with headers, body, and timeout
- `ApiResponse`: Parsed response with status, headers, and body

**Key Functions**:
- `post_json<T, R>(url, body: T) -> Result<R>`: POST with JSON serialization
- `with_retry<F>(operation: F, max_attempts: u32) -> Result<T>`: Exponential backoff retry

**Dependencies**:
- `reqwest` (0.11) with `json` feature - HTTP client
- `tokio` (1.0) with `full` feature - Async runtime
- `serde_json` (1.0) - JSON handling
- `core` - Error types

**Caution**: Using `tokio` with the `full` feature increases compile time and binary size; this is acceptable for an educational framework but production systems should use minimal feature sets.

### LLM Crate

**Purpose**: Implements unified interface for multiple LLM providers (OpenAI, Anthropic).

**Key Traits**:
- `LLMProvider`: Async trait with `send_message(&self, messages: &[Message]) -> Result<String>` and `stream_message(&self, messages: &[Message]) -> Result<Stream<String>>`

**Key Types**:
- `OpenAIProvider`: Implementation for OpenAI API (GPT-3.5, GPT-4)
- `AnthropicProvider`: Implementation for Anthropic API (Claude)
- `ProviderFactory`: Creates provider instances from configuration

**Key Functions**:
- `create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>>`: Factory function

**Dependencies**:
- `async-trait` (0.1) - Async trait support
- `communication` - HTTP client
- `config` - LLM configuration
- `core` - Message types and errors

**Resource**: Read "Async Rust" chapter in the Rust Book (https://rust-lang.github.io/async-book/) to understand async/await patterns used in the llm crate for API calls.


### Memory Crate

**Purpose**: Stores and retrieves conversation history with token-aware context management.

**Key Traits**:
- `MemoryStore`: Trait for different storage backends (in-memory, file, database)

**Key Types**:
- `InMemoryStore`: Vec-based implementation for MVP
- `ConversationHistory`: Wrapper with helper methods for common operations

**Key Functions**:
- `add_message(&mut self, message: Message)`: Append to history
- `get_recent(&self, limit: usize) -> Vec<Message>`: Retrieve last N messages
- `get_within_budget(&self, token_budget: usize) -> Vec<Message>`: Token-aware retrieval
- `clear(&mut self)`: Reset conversation

**Dependencies**:
- `core` - Message types
- `tiktoken-rs` (0.5) - Token counting for OpenAI models

**Resource**: Study the "Builder Pattern" in Rust Design Patterns (https://rust-unofficial.github.io/patterns/) to understand how ConversationHistory provides a fluent API in the memory crate.

### Tools Crate

**Purpose**: Defines tool interface and provides example tool implementations.

**Key Traits**:
- `Tool`: Trait with `name()`, `description()`, `parameters_schema()`, and `async execute(&self, params: Value) -> Result<Value>`

**Key Types**:
- `ToolRegistry`: HashMap-based registry for tool lookup
- `Calculator`: Example tool for arithmetic operations
- `FileReader`: Example tool for reading file contents
- `WebSearchStub`: Example tool that simulates web search

**Key Functions**:
- `register(&mut self, tool: Box<dyn Tool>)`: Add tool to registry
- `get(&self, name: &str) -> Option<&dyn Tool>`: Retrieve tool by name
- `list_tools(&self) -> Vec<ToolInfo>`: Get all available tools with descriptions

**Dependencies**:
- `async-trait` (0.1) - Async tool execution
- `serde_json` (1.0) - Parameter handling
- `core` - Error types


### Planner Crate

**Purpose**: Decomposes user goals into executable steps using LLM reasoning.

**Key Types**:
- `Plan`: Sequence of steps with reasoning
- `Step`: Enum with variants (ToolCall, Reasoning, Response)
- `ToolCall`: Structured tool invocation with name and parameters
- `Planner`: Orchestrates plan generation

**Key Functions**:
- `create_plan(&self, goal: &str, available_tools: &[ToolInfo]) -> Result<Plan>`: Generate plan from goal
- `validate_plan(&self, plan: &Plan, registry: &ToolRegistry) -> Result<()>`: Ensure tools exist

**Dependencies**:
- `llm` - LLM provider for reasoning
- `tools` - Tool information
- `memory` - Conversation context
- `core` - Base types

**Resource**: Read "ReAct: Synergizing Reasoning and Acting in Language Models" paper to understand the reasoning pattern implemented in the planner crate.

### Executor Crate

**Purpose**: Executes planned steps in sequence, managing tool invocations and result collection.

**Key Types**:
- `Executor`: Stateful executor with tool registry and memory
- `ExecutionResult`: Outcome of executing a plan (success, partial, failure)
- `StepResult`: Result of individual step execution

**Key Functions**:
- `execute_plan(&mut self, plan: Plan) -> Result<ExecutionResult>`: Run all steps
- `execute_step(&mut self, step: &Step) -> Result<StepResult>`: Run single step
- `handle_tool_call(&mut self, tool_call: &ToolCall) -> Result<Value>`: Invoke tool

**Dependencies**:
- `planner` - Plan types
- `tools` - Tool execution
- `memory` - Result storage
- `core` - Error handling


### Guardrails Crate

**Purpose**: Validates planned actions against safety constraints before execution.

**Key Traits**:
- `Guardrail`: Trait with `validate(&self, plan: &Plan) -> Result<()>` method

**Key Types**:
- `GuardrailRegistry`: Collection of active guardrails
- `FilePathGuardrail`: Restricts file operations to allowed directories
- `RateLimitGuardrail`: Enforces API call limits

**Key Functions**:
- `register(&mut self, guardrail: Box<dyn Guardrail>)`: Add guardrail
- `validate_all(&self, plan: &Plan) -> Result<()>`: Check all guardrails

**Dependencies**:
- `planner` - Plan types for validation
- `core` - Error types

**Resource**: Study "Type-Driven API Design in Rust" (https://www.youtube.com/watch?v=bnnacleqg6k) to understand how the guardrails crate uses Rust's type system to enforce safety at compile time.

### Rules Crate

**Purpose**: Customizes agent behavior through configurable rules that modify prompts and constraints.

**Key Traits**:
- `Rule`: Trait with `apply(&self, context: &mut PlanningContext)` method

**Key Types**:
- `RuleEngine`: Ordered collection of rules
- `ResponseLengthRule`: Limits response size
- `ToneRule`: Guides response style (formal, casual, technical)
- `PlanningContext`: Mutable context passed through rule chain

**Key Functions**:
- `add_rule(&mut self, rule: Box<dyn Rule>, priority: u32)`: Register rule with ordering
- `apply_all(&self, context: &mut PlanningContext)`: Execute rule chain

**Dependencies**:
- `core` - Base types


### CLI Crate

**Purpose**: Provides command-line interface for interacting with agents.

**Key Types**:
- `CliArgs`: Command-line argument parsing with clap
- `ReplMode`: Interactive conversation loop
- `SingleTurnMode`: One-shot question answering

**Key Functions**:
- `main()`: Entry point that initializes framework and starts chosen mode
- `run_repl(agent: Agent)`: Interactive loop with history display
- `run_single(agent: Agent, query: &str)`: Single query execution

**Dependencies**:
- `clap` (4.0) with `derive` feature - Argument parsing
- `rustyline` (12.0) - REPL with history and editing
- All framework crates for agent initialization

**Resource**: Read the clap documentation (https://docs.rs/clap) to understand derive-based CLI parsing used in the cli crate for argument handling.

### Examples Crate

**Purpose**: Demonstrates framework usage with three example agents.

**Example Agents**:
1. **Simple Chatbot** (`examples/chatbot.rs`): Basic conversation with no tools
2. **Research Assistant** (`examples/research.rs`): Uses web search and file reading tools
3. **File Manager** (`examples/file_manager.rs`): File operations with path guardrails

**Structure**:
- Each example is a separate binary in `examples/`
- Includes corresponding config file in `examples/configs/`
- README explaining what each demonstrates

**Dependencies**:
- All framework crates
- `tokio` for async main functions


## Data Models

### Message

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}
```

**Purpose**: Represents a single conversation turn. Defined in core crate and used throughout the framework.

### AgentConfig

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub llm: LLMConfig,
    pub memory: MemoryConfig,
    pub tools: Vec<String>,
    pub guardrails: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: usize,
}
```

**Purpose**: Configuration structure loaded from YAML/TOML files. Defined in config crate.

### Plan and Step

```rust
#[derive(Debug, Clone)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub enum Step {
    ToolCall(ToolCall),
    Reasoning(String),
    Response(String),
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: serde_json::Value,
}
```

**Purpose**: Represents the agent's execution plan. Defined in planner crate.


## Error Handling

### Error Strategy

The framework uses a layered error handling approach:

1. **Library crates** (core, config, llm, memory, tools, etc.) use `thiserror` to define typed errors
2. **Binary crates** (cli, examples) use `anyhow` for error propagation and context
3. All errors implement `std::error::Error` for interoperability

### Core Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("LLM provider error: {0}")]
    LLMProvider(String),
    
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolExecution { tool_name: String, reason: String },
    
    #[error("Guardrail violation: {0}")]
    GuardrailViolation(String),
    
    #[error("Memory error: {0}")]
    Memory(String),
    
    #[error("Planning error: {0}")]
    Planning(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Purpose**: Defined in core crate, this error type provides structured error information with context.

**Resource**: Read the thiserror documentation (https://docs.rs/thiserror) to understand derive-based error definitions used in the core crate for AgentError.


## Testing Strategy

### Unit Tests

Each crate includes unit tests for core functionality:
- **core**: Message creation, error conversion
- **config**: Configuration parsing, validation, merging
- **memory**: Message storage, retrieval, token counting
- **tools**: Tool registration, parameter validation
- **guardrails**: Validation logic for each guardrail
- **rules**: Rule application and ordering

**Pattern**: Use `#[cfg(test)]` modules in each source file with tests adjacent to implementation.

### Integration Tests

Integration tests in `tests/` directory verify cross-crate interactions:
- **llm_integration**: Real API calls to LLM providers (requires API keys)
- **agent_flow**: End-to-end agent execution with mocked LLM
- **tool_execution**: Tool registry with executor integration

**Pattern**: Use `tests/common/mod.rs` for shared test utilities and fixtures.

### Example Tests

Each example in the examples crate includes a test that verifies it compiles and runs:
- Ensures examples stay up-to-date with framework changes
- Provides smoke tests for major features

**Caution**: Integration tests that call real LLM APIs will incur costs and require API keys; use mocks for CI/CD pipelines and reserve real API tests for manual verification.

### Test Organization

```
ai-agent-framework/
├── core/
│   └── src/
│       ├── lib.rs          # Unit tests in #[cfg(test)] modules
│       └── message.rs      # Unit tests in #[cfg(test)] modules
├── tests/
│   ├── common/
│   │   └── mod.rs          # Shared test utilities
│   ├── llm_integration.rs  # Integration tests
│   └── agent_flow.rs       # Integration tests
└── examples/
    └── chatbot.rs          # Example with embedded test
```


## Daily Implementation Plan

### Day 1: Project Setup and Core Abstractions

**Objective**: Create the workspace structure and implement the core crate with fundamental types.

**Steps**:
1. Initialize cargo workspace with `cargo new --lib ai-agent-framework` and create workspace Cargo.toml
2. Create core crate with `cargo new --lib core` inside workspace
3. Define Message struct and Role enum in `core/src/message.rs` with serde derives
4. Define AgentError enum in `core/src/error.rs` using thiserror
5. Create `core/src/lib.rs` that re-exports Message, Role, AgentError, and Result type alias
6. Write unit tests for Message creation and error conversion
7. Build with `cargo build` and test with `cargo test`
8. Create root README.md explaining project structure and purpose

**Example Commands**:
```bash
cargo new --lib ai-agent-framework
cd ai-agent-framework
# Edit Cargo.toml to add [workspace] section
cargo new --lib core
cd core
# Add dependencies to Cargo.toml
cargo add serde --features derive
cargo add thiserror
cargo add chrono --features serde
cargo build
cargo test
```

**Example Cargo.toml (workspace root)**:
```toml
[workspace]
members = ["core"]
resolver = "2"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

**Blog Note**: "Day 1 established the workspace foundation with the core crate defining Message types and AgentError, giving us a solid base for building the agent framework."

**Progress Summary**: Created workspace structure and core crate with Message and error types. Next step is implementing configuration management.


### Day 2: Configuration Management

**Objective**: Implement the config crate to load and validate agent configuration from files and environment.

**Steps**:
1. Create config crate with `cargo new --lib config` and add to workspace members
2. Define AgentConfig, LLMConfig, and MemoryConfig structs in `config/src/lib.rs` with serde derives
3. Implement `load_from_file` function using serde_yaml to parse YAML config files
4. Implement `from_env` function to read configuration from environment variables (API_KEY, MODEL, etc.)
5. Implement `merge` function to combine file and environment configurations with env taking precedence
6. Add validation logic to ensure required fields (api_key, provider, model) are present
7. Write unit tests for parsing valid configs, handling missing fields, and merging logic
8. Create example config file `examples/configs/simple.yaml` with sample values

**Example Code Snippet**:
```rust
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub llm: LLMConfig,
    pub memory: MemoryConfig,
}

pub fn load_from_file(path: &Path) -> Result<AgentConfig> {
    let contents = std::fs::read_to_string(path)?;
    let config: AgentConfig = serde_yaml::from_str(&contents)?;
    validate(&config)?;
    Ok(config)
}
```

**Dependencies to Add**:
```bash
cd config
cargo add serde --features derive
cargo add serde_yaml
cargo add core --path ../core
```

**Blog Note**: "Day 2 built the configuration system that loads agent settings from YAML files and environment variables, with validation ensuring all required fields are present."

**Progress Summary**: Implemented config crate with file and environment loading. Next step is building HTTP communication utilities.


### Day 3: HTTP Communication Layer

**Objective**: Create the communication crate with HTTP client utilities and retry logic.

**Steps**:
1. Create communication crate with `cargo new --lib communication` and add to workspace
2. Define ApiClient struct wrapping reqwest::Client in `communication/src/client.rs`
3. Implement `post_json` method for JSON POST requests with proper error handling
4. Implement `with_retry` function with exponential backoff (initial delay 1s, max 3 attempts)
5. Add timeout configuration (default 30 seconds) to prevent hanging requests
6. Write unit tests using mockito or wiremock for HTTP mocking
7. Document retry behavior and timeout settings in module docs
8. Test with `cargo test` and verify retry logic with simulated failures

**Example Code Snippet**:
```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct ApiClient {
    client: Client,
    timeout: Duration,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            timeout: Duration::from_secs(30),
        }
    }
    
    pub async fn post_json<T, R>(&self, url: &str, body: &T) -> Result<R>
    where
        T: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let response = self.client
            .post(url)
            .json(body)
            .timeout(self.timeout)
            .send()
            .await?;
        Ok(response.json().await?)
    }
}
```

**Dependencies to Add**:
```bash
cd communication
cargo add reqwest --features json
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
cargo add core --path ../core
```

**Resource**: Read the reqwest documentation (https://docs.rs/reqwest) to understand async HTTP client patterns used in the communication crate for API calls.

**Blog Note**: "Day 3 created the communication layer with reqwest-based HTTP client, retry logic, and timeout handling for reliable API interactions."

**Progress Summary**: Built communication crate with HTTP utilities and retry logic. Next step is implementing LLM provider interfaces.


### Day 4: LLM Provider Interface

**Objective**: Define the LLMProvider trait and implement OpenAI provider.

**Steps**:
1. Create llm crate with `cargo new --lib llm` and add to workspace
2. Define LLMProvider async trait in `llm/src/provider.rs` with send_message method
3. Create OpenAI-specific request/response types in `llm/src/openai/types.rs`
4. Implement OpenAIProvider in `llm/src/openai/mod.rs` that calls chat completions API
5. Add proper error handling for API failures (auth, rate limits, network errors)
6. Write integration test that calls real OpenAI API (mark with #[ignore] for CI)
7. Create ProviderFactory in `llm/src/factory.rs` to instantiate providers from config
8. Document API key requirements and rate limiting considerations

**Example Code Snippet**:
```rust
use async_trait::async_trait;
use crate::core::Message;

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn send_message(&self, messages: &[Message]) -> Result<String>;
}

pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: ApiClient,
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        // Convert messages to OpenAI format and call API
        todo!()
    }
}
```

**Dependencies to Add**:
```bash
cd llm
cargo add async-trait
cargo add communication --path ../communication
cargo add config --path ../config
cargo add core --path ../core
```

**Resource**: Read the OpenAI API documentation (https://platform.openai.com/docs/api-reference/chat) to understand request/response formats implemented in the llm crate's OpenAIProvider.

**Blog Note**: "Day 4 defined the LLMProvider trait and implemented OpenAI integration, establishing a unified interface for language model interactions."

**Progress Summary**: Created LLM provider interface with OpenAI implementation. Next step is adding Anthropic provider support.


### Day 5: Anthropic Provider and Provider Factory

**Objective**: Add Anthropic (Claude) support and complete the provider factory.

**Steps**:
1. Create Anthropic-specific types in `llm/src/anthropic/types.rs` matching their API format
2. Implement AnthropicProvider in `llm/src/anthropic/mod.rs` with messages API
3. Handle Anthropic-specific features (system messages in separate field, different token counting)
4. Complete ProviderFactory to support both "openai" and "anthropic" provider strings
5. Add unit tests for provider factory with different configurations
6. Write integration test for Anthropic API (mark with #[ignore])
7. Update documentation with examples of both providers
8. Test switching between providers using different config files

**Example Code Snippet**:
```rust
pub fn create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(OpenAIProvider::new(config)?)),
        "anthropic" => Ok(Box::new(AnthropicProvider::new(config)?)),
        _ => Err(AgentError::Config(format!("Unknown provider: {}", config.provider))),
    }
}
```

**Example Config Files**:
```yaml
# openai.yaml
llm:
  provider: openai
  model: gpt-4
  api_key: ${OPENAI_API_KEY}
  temperature: 0.7

# anthropic.yaml
llm:
  provider: anthropic
  model: claude-3-sonnet-20240229
  api_key: ${ANTHROPIC_API_KEY}
  temperature: 0.7
```

**Resource**: Read the Anthropic API documentation (https://docs.anthropic.com/claude/reference) to understand message format differences implemented in the llm crate's AnthropicProvider.

**Caution**: Anthropic and OpenAI have different token counting methods and rate limits; the llm crate abstracts these differences but users should be aware when debugging API issues.

**Blog Note**: "Day 5 added Anthropic Claude support and completed the provider factory, enabling seamless switching between OpenAI and Anthropic models."

**Progress Summary**: Implemented Anthropic provider and factory pattern. Next step is building the memory system.


### Day 6: Memory System

**Objective**: Implement conversation memory with token-aware context management.

**Steps**:
1. Create memory crate with `cargo new --lib memory` and add to workspace
2. Define MemoryStore trait in `memory/src/store.rs` with add, get_recent, and clear methods
3. Implement InMemoryStore using Vec<Message> in `memory/src/in_memory.rs`
4. Add token counting using tiktoken-rs for OpenAI models
5. Implement `get_within_budget` method that returns messages fitting within token limit
6. Create ConversationHistory wrapper with helper methods in `memory/src/history.rs`
7. Write unit tests for message storage, retrieval, and token-aware filtering
8. Document token counting approach and limitations for different models

**Example Code Snippet**:
```rust
pub trait MemoryStore: Send + Sync {
    fn add_message(&mut self, message: Message);
    fn get_recent(&self, limit: usize) -> Vec<Message>;
    fn get_within_budget(&self, token_budget: usize) -> Vec<Message>;
    fn clear(&mut self);
}

pub struct InMemoryStore {
    messages: Vec<Message>,
}

impl MemoryStore for InMemoryStore {
    fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }
    
    fn get_recent(&self, limit: usize) -> Vec<Message> {
        self.messages.iter()
            .rev()
            .take(limit)
            .rev()
            .cloned()
            .collect()
    }
    
    fn get_within_budget(&self, token_budget: usize) -> Vec<Message> {
        // Implement token counting logic
        todo!()
    }
}
```

**Dependencies to Add**:
```bash
cd memory
cargo add tiktoken-rs
cargo add core --path ../core
```

**Resource**: Study the tiktoken-rs documentation (https://docs.rs/tiktoken-rs) to understand token counting for OpenAI models used in the memory crate's budget management.

**Blog Note**: "Day 6 built the memory system with token-aware context management, ensuring conversations stay within model context limits."

**Progress Summary**: Created memory crate with token-aware storage. Next step is implementing the tool system.


### Day 7: Tool System and Registry

**Objective**: Create the tool interface and implement example tools with a registry.

**Steps**:
1. Create tools crate with `cargo new --lib tools` and add to workspace
2. Define Tool async trait in `tools/src/tool.rs` with name, description, parameters_schema, and execute methods
3. Implement ToolRegistry in `tools/src/registry.rs` using HashMap for tool storage
4. Create Calculator tool in `tools/src/calculator.rs` for basic arithmetic operations
5. Create FileReader tool in `tools/src/file_reader.rs` for reading file contents
6. Create WebSearchStub tool in `tools/src/web_search.rs` that returns mock results
7. Write unit tests for each tool and registry operations
8. Document tool parameter schemas using JSON Schema format

**Example Code Snippet**:
```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> Value;
    async fn execute(&self, params: Value) -> Result<Value>;
}

pub struct Calculator;

#[async_trait]
impl Tool for Calculator {
    fn name(&self) -> &str { "calculator" }
    
    fn description(&self) -> &str {
        "Performs arithmetic operations (add, subtract, multiply, divide)"
    }
    
    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "operation": {"type": "string", "enum": ["add", "subtract", "multiply", "divide"]},
                "a": {"type": "number"},
                "b": {"type": "number"}
            },
            "required": ["operation", "a", "b"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        // Parse params and perform calculation
        todo!()
    }
}
```

**Dependencies to Add**:
```bash
cd tools
cargo add async-trait
cargo add serde_json
cargo add core --path ../core
```

**Resource**: Read JSON Schema documentation (https://json-schema.org/understanding-json-schema/) to understand parameter validation schemas used in the tools crate's Tool trait.

**Blog Note**: "Day 7 created the tool system with Calculator, FileReader, and WebSearchStub implementations, enabling agents to perform external actions."

**Progress Summary**: Implemented tool interface and example tools with registry. Next step is building the planner.


### Day 8: Planning System

**Objective**: Implement the planner that decomposes goals into executable steps using LLM reasoning.

**Steps**:
1. Create planner crate with `cargo new --lib planner` and add to workspace
2. Define Plan and Step types in `planner/src/types.rs` with ToolCall, Reasoning, and Response variants
3. Implement Planner struct in `planner/src/planner.rs` that uses LLM to generate plans
4. Create system prompt template that instructs LLM on plan format and available tools
5. Implement plan parsing logic to extract structured steps from LLM response
6. Add validation to ensure tool calls reference existing tools in registry
7. Write unit tests with mocked LLM responses
8. Test with real LLM to verify plan generation quality

**Example Code Snippet**:
```rust
#[derive(Debug, Clone)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub reasoning: String,
}

#[derive(Debug, Clone)]
pub enum Step {
    ToolCall(ToolCall),
    Reasoning(String),
    Response(String),
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub parameters: Value,
}

pub struct Planner {
    llm: Box<dyn LLMProvider>,
    memory: Box<dyn MemoryStore>,
}

impl Planner {
    pub async fn create_plan(&self, goal: &str, available_tools: &[ToolInfo]) -> Result<Plan> {
        let system_prompt = self.build_system_prompt(available_tools);
        let messages = vec![
            Message::system(system_prompt),
            Message::user(goal),
        ];
        let response = self.llm.send_message(&messages).await?;
        self.parse_plan(&response)
    }
}
```

**Dependencies to Add**:
```bash
cd planner
cargo add llm --path ../llm
cargo add tools --path ../tools
cargo add memory --path ../memory
cargo add core --path ../core
cargo add serde_json
```

**Resource**: Read "Chain-of-Thought Prompting Elicits Reasoning in Large Language Models" paper to understand the reasoning patterns implemented in the planner crate's system prompts.

**Blog Note**: "Day 8 built the planning system that uses LLM reasoning to decompose user goals into structured, executable steps."

**Progress Summary**: Created planner with LLM-based task decomposition. Next step is implementing the executor.


### Day 9: Execution System

**Objective**: Build the executor that runs planned steps and manages tool invocations.

**Steps**:
1. Create executor crate with `cargo new --lib executor` and add to workspace
2. Define ExecutionResult and StepResult types in `executor/src/types.rs`
3. Implement Executor struct in `executor/src/executor.rs` with tool registry and memory
4. Implement `execute_plan` method that iterates through steps sequentially
5. Implement `execute_step` method with pattern matching on Step variants
6. Implement `handle_tool_call` method that invokes tools and captures results
7. Add result storage to memory after each step for context in subsequent steps
8. Write unit tests with mocked tools and verify execution flow

**Example Code Snippet**:
```rust
pub struct Executor {
    tools: ToolRegistry,
    memory: Box<dyn MemoryStore>,
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub success: bool,
    pub final_response: String,
    pub step_results: Vec<StepResult>,
}

impl Executor {
    pub async fn execute_plan(&mut self, plan: Plan) -> Result<ExecutionResult> {
        let mut step_results = Vec::new();
        
        for step in plan.steps {
            let result = self.execute_step(&step).await?;
            step_results.push(result.clone());
            
            // Add result to memory for context
            self.memory.add_message(Message::assistant(result.output.clone()));
        }
        
        Ok(ExecutionResult {
            success: true,
            final_response: self.build_final_response(&step_results),
            step_results,
        })
    }
    
    async fn execute_step(&mut self, step: &Step) -> Result<StepResult> {
        match step {
            Step::ToolCall(tool_call) => self.handle_tool_call(tool_call).await,
            Step::Reasoning(text) => Ok(StepResult::reasoning(text.clone())),
            Step::Response(text) => Ok(StepResult::response(text.clone())),
        }
    }
}
```

**Dependencies to Add**:
```bash
cd executor
cargo add planner --path ../planner
cargo add tools --path ../tools
cargo add memory --path ../memory
cargo add core --path ../core
```

**Blog Note**: "Day 9 implemented the executor that runs planned steps sequentially, invoking tools and managing results through the memory system."

**Progress Summary**: Built executor with step-by-step execution logic. Next step is adding safety guardrails.


### Day 10: Guardrails System

**Objective**: Implement safety guardrails that validate plans before execution.

**Steps**:
1. Create guardrails crate with `cargo new --lib guardrails` and add to workspace
2. Define Guardrail trait in `guardrails/src/guardrail.rs` with validate method
3. Implement GuardrailRegistry in `guardrails/src/registry.rs` for managing multiple guardrails
4. Create FilePathGuardrail in `guardrails/src/file_path.rs` that restricts file operations to allowed directories
5. Create RateLimitGuardrail in `guardrails/src/rate_limit.rs` that tracks and limits API calls
6. Implement `validate_all` method that checks plan against all registered guardrails
7. Write unit tests for each guardrail with valid and invalid plans
8. Document guardrail configuration and customization options

**Example Code Snippet**:
```rust
pub trait Guardrail: Send + Sync {
    fn name(&self) -> &str;
    fn validate(&self, plan: &Plan) -> Result<()>;
}

pub struct FilePathGuardrail {
    allowed_paths: Vec<PathBuf>,
}

impl Guardrail for FilePathGuardrail {
    fn name(&self) -> &str { "file_path" }
    
    fn validate(&self, plan: &Plan) -> Result<()> {
        for step in &plan.steps {
            if let Step::ToolCall(tool_call) = step {
                if tool_call.tool_name == "file_reader" {
                    let path = self.extract_path(&tool_call.parameters)?;
                    if !self.is_allowed(&path) {
                        return Err(AgentError::GuardrailViolation(
                            format!("File path not allowed: {}", path.display())
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct GuardrailRegistry {
    guardrails: Vec<Box<dyn Guardrail>>,
}

impl GuardrailRegistry {
    pub fn validate_all(&self, plan: &Plan) -> Result<()> {
        for guardrail in &self.guardrails {
            guardrail.validate(plan)?;
        }
        Ok(())
    }
}
```

**Dependencies to Add**:
```bash
cd guardrails
cargo add planner --path ../planner
cargo add core --path ../core
```

**Resource**: Study "Rust API Guidelines" (https://rust-lang.github.io/api-guidelines/) to understand trait design patterns used in the guardrails crate's Guardrail trait.

**Blog Note**: "Day 10 added safety guardrails with file path restrictions and rate limiting, preventing agents from performing unauthorized actions."

**Progress Summary**: Implemented guardrails system with validation logic. Next step is building the rules engine.


### Day 11: Rules Engine

**Objective**: Create a rules system for customizing agent behavior through prompt modification.

**Steps**:
1. Create rules crate with `cargo new --lib rules` and add to workspace
2. Define Rule trait in `rules/src/rule.rs` with apply method that modifies PlanningContext
3. Define PlanningContext struct in `rules/src/context.rs` with system_prompt, constraints, and metadata
4. Implement RuleEngine in `rules/src/engine.rs` with priority-based rule ordering
5. Create ResponseLengthRule in `rules/src/response_length.rs` that adds length constraints
6. Create ToneRule in `rules/src/tone.rs` that modifies system prompt for style guidance
7. Write unit tests verifying rule application and ordering
8. Document how to create custom rules and configure rule priorities

**Example Code Snippet**:
```rust
pub trait Rule: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> u32;
    fn apply(&self, context: &mut PlanningContext);
}

pub struct PlanningContext {
    pub system_prompt: String,
    pub constraints: Vec<String>,
    pub metadata: HashMap<String, String>,
}

pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    pub fn apply_all(&self, context: &mut PlanningContext) {
        let mut sorted_rules = self.rules.iter().collect::<Vec<_>>();
        sorted_rules.sort_by_key(|r| r.priority());
        
        for rule in sorted_rules {
            rule.apply(context);
        }
    }
}

pub struct ResponseLengthRule {
    max_words: usize,
}

impl Rule for ResponseLengthRule {
    fn name(&self) -> &str { "response_length" }
    fn priority(&self) -> u32 { 100 }
    
    fn apply(&self, context: &mut PlanningContext) {
        context.constraints.push(
            format!("Keep responses under {} words", self.max_words)
        );
    }
}
```

**Dependencies to Add**:
```bash
cd rules
cargo add core --path ../core
```

**Blog Note**: "Day 11 built the rules engine that customizes agent behavior through priority-based prompt modifications and constraints."

**Progress Summary**: Created rules system with customizable behavior. Next step is building the CLI interface.


### Day 12: Command-Line Interface

**Objective**: Build a CLI for interacting with agents in both single-turn and REPL modes.

**Steps**:
1. Create cli crate with `cargo new --bin cli` and add to workspace
2. Define CliArgs struct in `cli/src/args.rs` using clap derive macros for argument parsing
3. Implement agent initialization in `cli/src/agent.rs` that wires together all components
4. Implement single-turn mode in `cli/src/single.rs` for one-shot queries
5. Implement REPL mode in `cli/src/repl.rs` using rustyline for interactive conversations
6. Add conversation history display in REPL with colored output
7. Implement graceful error handling and user-friendly error messages
8. Test both modes with example queries and verify component integration

**Example Code Snippet**:
```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ai-agent")]
#[command(about = "Educational AI Agent Framework", long_about = None)]
struct CliArgs {
    /// Path to configuration file
    #[arg(short, long)]
    config: PathBuf,
    
    /// Run in single-turn mode with this query
    #[arg(short, long)]
    query: Option<String>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    
    // Load config and initialize agent
    let config = config::load_from_file(&args.config)?;
    let agent = Agent::new(config).await?;
    
    match args.query {
        Some(query) => single::run(&agent, &query).await?,
        None => repl::run(agent).await?,
    }
    
    Ok(())
}
```

**Example REPL Implementation**:
```rust
use rustyline::Editor;

pub async fn run(mut agent: Agent) -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new()?;
    
    println!("AI Agent REPL - Type 'exit' to quit");
    
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim() == "exit" { break; }
                
                match agent.process(&line).await {
                    Ok(response) => println!("{}", response),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => break,
        }
    }
    
    Ok(())
}
```

**Dependencies to Add**:
```bash
cd cli
cargo add clap --features derive
cargo add rustyline
cargo add tokio --features full
cargo add anyhow
cargo add colored
# Add all framework crates
cargo add core --path ../core
cargo add config --path ../config
cargo add llm --path ../llm
cargo add memory --path ../memory
cargo add tools --path ../tools
cargo add planner --path ../planner
cargo add executor --path ../executor
cargo add guardrails --path ../guardrails
cargo add rules --path ../rules
```

**Resource**: Read the clap documentation (https://docs.rs/clap) to understand derive-based CLI parsing used in the cli crate for argument handling.

**Blog Note**: "Day 12 created the CLI with both single-turn and interactive REPL modes, providing a user-friendly interface for agent interactions."

**Progress Summary**: Built CLI with REPL and single-turn modes. Next step is creating example agents.


### Day 13: Example Agents

**Objective**: Create three example agents demonstrating different framework capabilities.

**Steps**:
1. Create examples directory and add example binaries to workspace
2. Create simple chatbot example in `examples/chatbot.rs` with no tools, just conversation
3. Create research assistant example in `examples/research.rs` with web search and file reading
4. Create file manager example in `examples/file_manager.rs` with file operations and path guardrails
5. Create corresponding config files in `examples/configs/` for each example
6. Write README for each example explaining what it demonstrates and how to run it
7. Add example-specific documentation showing expected interactions
8. Test each example with sample queries to verify functionality

**Example Chatbot** (`examples/chatbot.rs`):
```rust
use ai_agent_framework::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load_from_file("examples/configs/chatbot.yaml")?;
    
    let llm = llm::create_provider(&config.llm)?;
    let memory = memory::InMemoryStore::new();
    
    // Simple agent with no tools
    let mut agent = Agent::builder()
        .llm(llm)
        .memory(memory)
        .build();
    
    println!("Simple Chatbot - Type your message:");
    // REPL loop here
    
    Ok(())
}
```

**Example Research Assistant** (`examples/research.rs`):
```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load_from_file("examples/configs/research.yaml")?;
    
    let mut tools = tools::ToolRegistry::new();
    tools.register(Box::new(tools::WebSearchStub));
    tools.register(Box::new(tools::FileReader::new()));
    
    let agent = Agent::builder()
        .llm(llm::create_provider(&config.llm)?)
        .memory(memory::InMemoryStore::new())
        .tools(tools)
        .build();
    
    // Demonstrate research workflow
    Ok(())
}
```

**Example Config** (`examples/configs/chatbot.yaml`):
```yaml
llm:
  provider: openai
  model: gpt-3.5-turbo
  api_key: ${OPENAI_API_KEY}
  temperature: 0.7
  max_tokens: 500

memory:
  max_messages: 20
  token_budget: 2000
```

**Blog Note**: "Day 13 created three example agents (chatbot, research assistant, file manager) demonstrating progressive framework capabilities from simple to complex."

**Progress Summary**: Built example agents with documentation. Next step is adding integration tests and polish.


### Day 14: Testing, Documentation, and Polish

**Objective**: Add integration tests, improve documentation, and prepare for publication.

**Steps**:
1. Create `tests/` directory at workspace root for integration tests
2. Write integration test in `tests/agent_flow.rs` that tests end-to-end agent execution with mocked LLM
3. Write integration test in `tests/tool_execution.rs` verifying tool registry and executor integration
4. Create test utilities in `tests/common/mod.rs` for shared mocks and fixtures
5. Update root README.md with comprehensive project overview, architecture diagram, and getting started guide
6. Add inline documentation to all public APIs using rustdoc comments
7. Run `cargo doc --open` to verify documentation quality
8. Create CONTRIBUTING.md with guidelines for extending the framework

**Example Integration Test** (`tests/agent_flow.rs`):
```rust
use ai_agent_framework::*;

struct MockLLM {
    responses: Vec<String>,
    call_count: std::sync::Arc<std::sync::Mutex<usize>>,
}

#[async_trait::async_trait]
impl llm::LLMProvider for MockLLM {
    async fn send_message(&self, _messages: &[Message]) -> Result<String> {
        let mut count = self.call_count.lock().unwrap();
        let response = self.responses[*count].clone();
        *count += 1;
        Ok(response)
    }
}

#[tokio::test]
async fn test_agent_with_tool_execution() {
    let mock_llm = MockLLM {
        responses: vec![
            r#"{"steps": [{"tool_call": {"tool_name": "calculator", "parameters": {"operation": "add", "a": 5, "b": 3}}}]}"#.to_string(),
        ],
        call_count: std::sync::Arc::new(std::sync::Mutex::new(0)),
    };
    
    let mut tools = tools::ToolRegistry::new();
    tools.register(Box::new(tools::Calculator));
    
    let planner = planner::Planner::new(Box::new(mock_llm), memory::InMemoryStore::new());
    let plan = planner.create_plan("What is 5 + 3?", &tools.list_tools()).await.unwrap();
    
    let mut executor = executor::Executor::new(tools, memory::InMemoryStore::new());
    let result = executor.execute_plan(plan).await.unwrap();
    
    assert!(result.success);
}
```

**Root README Structure**:
```markdown
# AI Agent Framework

Educational Rust framework for building autonomous AI agents.

## Architecture

[Diagram showing component relationships]

## Getting Started

1. Clone the repository
2. Set up API keys: `export OPENAI_API_KEY=...`
3. Run example: `cargo run --example chatbot`

## Components

- **core**: Fundamental types and errors
- **config**: Configuration management
- **llm**: LLM provider interfaces
- **memory**: Conversation storage
- **tools**: Tool system and implementations
- **planner**: Task decomposition
- **executor**: Step execution
- **guardrails**: Safety validation
- **rules**: Behavior customization
- **cli**: Command-line interface

## Blog Series

This framework was built as part of a 14-day blog series...
```

**Commands to Run**:
```bash
# Run all tests
cargo test --workspace

# Generate documentation
cargo doc --open --no-deps

# Check code quality
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Build release binaries
cargo build --release
```

**Resource**: Read "The Rust Programming Language" book chapter on testing (https://doc.rust-lang.org/book/ch11-00-testing.html) to understand integration test patterns used in the tests directory.

**Blog Note**: "Day 14 added comprehensive integration tests, polished documentation, and prepared the framework for publication with examples and contribution guidelines."

**Progress Summary**: Completed testing, documentation, and polish. The framework is now ready for publication and use in educational contexts.


## Additional Resources and Learning Materials

### Rust-Specific Resources

1. **The Rust Book** (https://doc.rust-lang.org/book/) - Essential reading for understanding ownership, traits, and error handling used throughout the core crate
2. **Async Rust Book** (https://rust-lang.github.io/async-book/) - Required for understanding async/await patterns in the llm and communication crates
3. **Rust API Guidelines** (https://rust-lang.github.io/api-guidelines/) - Informs trait design in tools, guardrails, and rules crates
4. **Rust Design Patterns** (https://rust-unofficial.github.io/patterns/) - Builder and strategy patterns used in memory and planner crates

### AI and Agent Architecture Resources

1. **ReAct Paper** (https://arxiv.org/abs/2210.03629) - "ReAct: Synergizing Reasoning and Acting in Language Models" - Foundation for the planner crate's reasoning approach
2. **Chain-of-Thought Paper** (https://arxiv.org/abs/2201.11903) - Explains prompting strategies used in the planner crate's system prompts
3. **LangChain Documentation** (https://docs.langchain.com/) - Reference architecture for agent frameworks, though this implementation is simpler and more educational
4. **OpenAI Function Calling Guide** (https://platform.openai.com/docs/guides/function-calling) - Relevant to tool integration in the tools and executor crates

### Crate-Specific Documentation

1. **tokio** (https://docs.rs/tokio) - Async runtime used in communication and llm crates for non-blocking API calls
2. **reqwest** (https://docs.rs/reqwest) - HTTP client in communication crate for API requests
3. **clap** (https://docs.rs/clap) - CLI parsing in cli crate for argument handling
4. **thiserror** (https://docs.rs/thiserror) - Error definitions in core crate for structured error types
5. **serde** (https://docs.rs/serde) - Serialization throughout config, core, and llm crates
6. **tiktoken-rs** (https://docs.rs/tiktoken-rs) - Token counting in memory crate for context management

### Trade-offs and Cautions

1. **Tokio Full Feature Set**: Using `tokio` with `full` features increases compile time and binary size; acceptable for education but production should use minimal features
2. **Real API Testing**: Integration tests calling real LLM APIs incur costs; use mocks for CI/CD and reserve real tests for manual verification
3. **Token Counting Accuracy**: tiktoken-rs is OpenAI-specific; Anthropic and other providers use different tokenization, affecting memory crate's budget calculations
4. **Synchronous Tool Execution**: Tools use async trait but many operations could be sync; this design choice prioritizes consistency over performance
5. **In-Memory Storage**: The memory crate's InMemoryStore loses data on restart; production systems need persistent storage implementations
6. **Error Granularity**: AgentError in core crate provides broad categories; production systems may need more specific error types per component
7. **Plan Parsing Brittleness**: The planner crate parses LLM responses as structured data; LLM output variability can cause parsing failures requiring robust error handling


## Extension Points and Future Enhancements

### Potential Extensions for Blog Series Continuation

1. **Streaming Responses**: Implement streaming in the llm crate's LLMProvider trait to show real-time token generation
2. **Persistent Memory**: Add file-based or database-backed MemoryStore implementations in the memory crate
3. **Advanced Tools**: Create more sophisticated tools (actual web search with APIs, database queries, code execution sandboxes)
4. **Parallel Execution**: Modify executor crate to support parallel step execution for independent operations
5. **Plan Refinement**: Add feedback loops where executor results can trigger plan modifications in the planner crate
6. **Observability**: Add structured logging and tracing throughout all crates for debugging and monitoring
7. **Web Interface**: Create a web-based UI as alternative to the cli crate using axum or actix-web
8. **Multi-Agent Systems**: Extend the framework to support multiple agents collaborating on tasks
9. **Fine-tuning Integration**: Add support for fine-tuned models and custom model endpoints in the llm crate
10. **Vector Memory**: Implement semantic search over conversation history using embeddings in the memory crate

### Configuration Extensions

The config crate can be extended to support:
- Multiple agent profiles in a single config file
- Hot-reloading of configuration without restart
- Schema validation for config files
- Config generation from interactive prompts

### Tool System Extensions

The tools crate provides a foundation for:
- Tool composition (tools that call other tools)
- Tool versioning and deprecation
- Tool usage analytics and optimization
- Dynamic tool loading from plugins
- Tool parameter validation using JSON Schema

### Safety Extensions

The guardrails crate can be enhanced with:
- Content filtering for inappropriate outputs
- Budget tracking for API costs
- Time-based execution limits
- Audit logging of all agent actions
- User approval workflows for sensitive operations


## Summary and Next Steps

### What We've Designed

This design document provides a complete 14-day implementation plan for an educational AI agent framework in Rust. The architecture follows clean separation of concerns with 12 crates organized into three layers:

1. **Foundation** (Days 1-3): core, config, communication
2. **Capabilities** (Days 4-7): llm, memory, tools  
3. **Intelligence** (Days 8-14): planner, executor, guardrails, rules, cli, examples

Each day includes:
- Clear objective and concrete steps
- Example code snippets that compile conceptually
- Specific commands to run
- Dependencies to add
- Learning resources with rationale
- Blog note for publication
- Progress summary

### Key Design Decisions

1. **Workspace Structure**: Separate crates for each concern enable independent learning and testing
2. **Trait-Based Abstractions**: LLMProvider, Tool, Guardrail, and Rule traits allow extensibility
3. **Async Throughout**: Consistent async/await usage prepares for real-world I/O operations
4. **Error Handling**: thiserror for libraries, anyhow for binaries provides ergonomic error management
5. **Educational Focus**: Prioritizes clarity and learning over performance optimization

### Requirements Coverage

This design addresses all 12 requirements from the requirements document:
- **Req 1**: Workspace structure with separated crates
- **Req 2**: Configuration management with file and env support
- **Req 3**: Unified LLM interface with multiple providers
- **Req 4**: Memory system with token-aware context
- **Req 5**: Tool system with registry and example implementations
- **Req 6**: Planning system with LLM-based decomposition
- **Req 7**: Executor for step-by-step execution
- **Req 8**: Guardrails for safety validation
- **Req 9**: Rules engine for behavior customization
- **Req 10**: CLI with REPL and single-turn modes
- **Req 11**: Three example agents demonstrating features
- **Req 12**: Comprehensive error handling across all components

### Ready for Implementation

The design is complete and ready for implementation. Each day's tasks are:
- Concrete and actionable
- Testable with clear success criteria
- Documented with examples and resources
- Suitable for blog post content

The next phase is to begin Day 1 implementation: creating the workspace and core crate.
