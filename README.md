<img width="529" height="311" alt="Screenshot 2025-11-24 at 18 50 39" src="https://github.com/user-attachments/assets/2d2e061a-affa-4b5b-9ff3-ae47b37af91c" />

# Samurai AI Agent Framework

An educational Rust framework for building autonomous AI agents with memory, planning, tool execution, and safety guardrails.

## Overview

This framework demonstrates how to build production-quality AI agents in Rust, with clear architectural boundaries and incremental complexity. It's designed as a learning resource with a modular workspace structure where each component can be understood independently.

### Motivation

Building AI agents requires orchestrating multiple complex systems: language models, memory management, tool execution, and safety constraints. This framework provides a clear, well-structured implementation that:

- **Teaches by Example**: Each crate demonstrates a specific architectural pattern (traits, async execution, error handling)
- **Production-Ready Patterns**: Uses industry-standard libraries and follows Rust best practices
- **Incremental Learning**: Components can be studied independently, building from simple to complex
- **Safety First**: Demonstrates how to build guardrails and validation into AI systems
- **Extensible Design**: Easy to add new LLM providers, tools, and safety rules

This project serves as both a functional framework and a comprehensive tutorial on building AI systems in Rust.

## Architecture

The framework is organized into three layers, with clear separation of concerns and minimal coupling:

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI & Examples                           │
│                    (User Interface Layer)                       │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                       Intelligence Layer                        │
│    ┌──────────┐  ┌──────────┐  ┌───────────┐  ┌──────────┐      │
│    │ Planner  │  │ Executor │  │Guardrails │  │  Rules   │      │
│    └──────────┘  └──────────┘  └───────────┘  └──────────┘      │
│         │              │               │              │         │
└────--───┼──────────────┼───────────────┼──────────────┼─────────┘
          │              │               │              │
┌──────--─┴──────────────┴───────────────┴──────────────┴─────────┐
│                      Capability Layer                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                       │
│  │   LLM    │  │  Memory  │  │  Tools   │                       │
│  └──────────┘  └──────────┘  └──────────┘                       │
│       │              │               │                          │
└───────┼──────────────┼───────────────┼──────────────────────────┘
        │              │               │
┌───────┴──────────────┴───────────────┴──────────────────────────┐
│                      Foundation Layer                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐                   │
│  │   Core   │  │  Config  │  │Communication │                   │
│  └──────────┘  └──────────┘  └──────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

### Foundation Layer
- **core** - Fundamental types (Message, Role, AgentError) and error handling
- **config** - Configuration management from files and environment variables
- **communication** - HTTP client utilities with retry logic and timeout handling

### Capability Layer
- **llm** - LLM provider interfaces (OpenAI, Anthropic) with unified API
- **memory** - Conversation storage with token-aware context management
- **tools** - Tool system with registry and example implementations (Calculator, FileReader, WebSearch)

### Intelligence Layer
- **planner** - Task decomposition using LLM reasoning (ReAct pattern)
- **executor** - Step-by-step execution of plans with tool invocation
- **guardrails** - Safety validation before execution (file paths, rate limits)
- **rules** - Behavior customization through prompt modification

### Interface Layer
- **cli** - Command-line interface (REPL and single-turn modes)
- **examples** - Example agents demonstrating framework capabilities

## Project Structure

```
ai-agent-framework/
├── Cargo.toml              # Workspace manifest
├── README.md               # This file
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

## Getting Started

### Prerequisites

- **Rust**: Version 1.70 or later ([Install Rust](https://rustup.rs/))
- **LLM Provider** (choose one):
  - **Ollama** (Recommended for local development) - [Install Ollama](https://ollama.ai)
  - **OpenAI** - API key from [OpenAI API Keys](https://platform.openai.com/api-keys)
  - **Anthropic** - API key from [Anthropic API Keys](https://console.anthropic.com/settings/keys)

### Quick Start

#### Option A: Using Ollama (Local, No API Key Required)

**With Docker (Recommended)**:
```bash
git clone https://github.com/netologist/samurai ai-agent-framework
cd ai-agent-framework

# One-command setup (starts container + pulls model)
make ollama-setup
# Or: ./scripts/setup-ollama.sh

# Run the chatbot
make run-ollama
```

**Native Installation**:
```bash
# Install Ollama from https://ollama.ai
ollama pull llama2

# Clone and run
git clone https://github.com/netologist/samurai ai-agent-framework
cd ai-agent-framework
cargo build --workspace
cargo run --example ollama_chatbot
```

#### Option B: Using OpenAI or Anthropic

1. **Clone the repository**:
```bash
git clone https://github.com/netologist/samurai ai-agent-framework
cd ai-agent-framework
```

2. **Set up your API key**:
```bash
# For OpenAI (GPT-3.5, GPT-4)
export OPENAI_API_KEY="sk-..."

# OR for Anthropic (Claude)
export ANTHROPIC_API_KEY="sk-ant-..."
```

3. **Build the project**:
```bash
cargo build --workspace
```

4. **Run your first agent**:
```bash
cargo run --example chatbot
```

You should see an interactive prompt where you can chat with the AI agent!

### Running Examples

The framework includes four example agents that demonstrate different capabilities:

#### 1. Ollama Chatbot (No API Key Required!)
Conversational agent using local open-source models via Ollama.

```bash
cargo run --example ollama_chatbot
```

**What it demonstrates**:
- Running agents completely locally
- Using open-source models (llama2, mistral, etc.)
- No API costs or internet dependency

**Prerequisites**: Ollama installed with a model pulled (e.g., `ollama pull llama2`)

#### 2. Simple Chatbot
Basic conversational agent with memory but no tools.

```bash
cargo run --example chatbot
```

**What it demonstrates**:
- LLM integration (OpenAI/Anthropic/Ollama)
- Conversation memory
- Multi-turn interactions

#### 3. Research Assistant
Agent with web search and file reading capabilities.

```bash
cargo run --example research
```

**What it demonstrates**:
- Tool integration and execution
- Multi-step planning
- Tool result handling

#### 4. File Manager
Agent with file operations and safety guardrails.

```bash
cargo run --example file_manager
```

**What it demonstrates**:
- Guardrail validation
- File path restrictions
- Safe tool execution

### Basic Usage

Here's a minimal example of creating an agent:

```rust
use ai_agent_framework::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = config::load_from_file("config.yaml")?;
    
    // Initialize components
    let llm = llm::create_provider(&config.llm)?;
    let memory = memory::InMemoryStore::new();
    let tools = tools::ToolRegistry::new();
    
    // Create planner and executor
    let planner = planner::Planner::new(llm.clone(), memory.clone());
    let executor = executor::Executor::new(tools, memory);
    
    // Process a query
    let plan = planner.create_plan("What is 2 + 2?", &tools.list_tools()).await?;
    let result = executor.execute_plan(plan).await?;
    
    println!("Result: {}", result.final_response);
    Ok(())
}
```

### Configuration

Create a `config.yaml` file:

```yaml
llm:
  provider: openai  # or "anthropic"
  model: gpt-4
  api_key: ${OPENAI_API_KEY}
  temperature: 0.7
  max_tokens: 2000

memory:
  max_messages: 100
  token_budget: 4000

tools:
  - calculator
  - file_reader

guardrails:
  - file_path
  - rate_limit
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p core

# Run integration tests
cargo test --test agent_flow

# Run with output
cargo test -- --nocapture
```

## Crate Documentation

Each crate has a specific responsibility and can be understood independently:

### Core Crate (`core/`)
**Purpose**: Fundamental types and error handling used throughout the framework.

**Key Types**:
- `Message` - Represents conversation turns with role, content, and timestamp
- `Role` - Enum for System, User, and Assistant roles
- `AgentError` - Common error type with structured error information using thiserror
- `Result<T>` - Type alias for `std::result::Result<T, AgentError>`

**Dependencies**: `serde`, `thiserror`, `chrono`

**When to use**: Import core types when building any framework component.

---

### Config Crate (`config/`)
**Purpose**: Configuration management from files and environment variables.

**Key Functions**:
- `load_from_file(path)` - Parse YAML/TOML configuration files
- `from_env()` - Build configuration from environment variables
- `merge(file, env)` - Combine file and environment configs (env takes precedence)

**Configuration Structure**:
- `AgentConfig` - Top-level configuration
- `LLMConfig` - Provider settings (provider, model, api_key, temperature, max_tokens)
- `MemoryConfig` - Memory settings (max_messages, token_budget)

**Dependencies**: `serde`, `serde_yaml`, `core`

**When to use**: Load configuration at application startup before initializing other components.

---

### Communication Crate (`communication/`)
**Purpose**: HTTP client utilities with retry logic and error handling.

**Key Types**:
- `ApiClient` - Wrapper around reqwest with timeout and retry support
- `with_retry()` - Exponential backoff retry function (max 3 attempts)

**Features**:
- 30-second default timeout
- Automatic retry on network errors and 5xx responses
- JSON serialization/deserialization
- Structured error conversion

**Dependencies**: `reqwest`, `tokio`, `serde_json`, `core`

**When to use**: Use for all HTTP API calls to LLM providers.

---

### LLM Crate (`llm/`)
**Purpose**: Unified interface for multiple LLM providers.

**Key Trait**:
- `LLMProvider` - Async trait with `send_message(&self, messages: &[Message]) -> Result<String>`

**Implementations**:
- `OpenAIProvider` - OpenAI API (GPT-3.5, GPT-4)
- `AnthropicProvider` - Anthropic API (Claude models)
- `OllamaProvider` - Local Ollama server (llama2, mistral, phi, etc.)

**Factory**:
- `create_provider(config)` - Creates provider instance from configuration

**Supported Providers**:
- **OpenAI**: Cloud-based, requires API key, supports GPT models
- **Anthropic**: Cloud-based, requires API key, supports Claude models
- **Ollama**: Local execution, no API key needed, supports open-source models

**Dependencies**: `async-trait`, `communication`, `config`, `core`

**When to use**: Initialize at startup and use for all LLM interactions.

---

### Memory Crate (`memory/`)
**Purpose**: Conversation storage with token-aware context management.

**Key Trait**:
- `MemoryStore` - Trait for different storage backends

**Implementations**:
- `InMemoryStore` - Vec-based storage for MVP
- `ConversationHistory` - Wrapper with helper methods

**Key Methods**:
- `add_message(message)` - Append to conversation history
- `get_recent(limit)` - Retrieve last N messages
- `get_within_budget(tokens)` - Token-aware retrieval
- `clear()` - Reset conversation

**Dependencies**: `tiktoken-rs`, `core`

**When to use**: Store all conversation turns and retrieve context for LLM calls.

---

### Tools Crate (`tools/`)
**Purpose**: Extensible tool system for agent capabilities.

**Key Trait**:
- `Tool` - Async trait with `name()`, `description()`, `parameters_schema()`, `execute(params)`

**Registry**:
- `ToolRegistry` - HashMap-based tool storage and lookup

**Built-in Tools**:
- `Calculator` - Arithmetic operations (add, subtract, multiply, divide)
- `FileReader` - Read file contents with error handling
- `WebSearchStub` - Mock web search for demonstration

**Dependencies**: `async-trait`, `serde_json`, `core`

**When to use**: Register tools at startup; executor invokes them during plan execution.

---

### Planner Crate (`planner/`)
**Purpose**: LLM-based task decomposition using ReAct pattern.

**Key Types**:
- `Plan` - Sequence of steps with reasoning
- `Step` - Enum: ToolCall, Reasoning, Response
- `ToolCall` - Structured tool invocation (name + parameters)
- `Planner` - Orchestrates plan generation

**Key Methods**:
- `create_plan(goal, tools)` - Generate plan from user goal
- `validate_plan(plan, registry)` - Ensure all tools exist

**Dependencies**: `llm`, `tools`, `memory`, `core`

**When to use**: Convert user queries into executable plans before execution.

---

### Executor Crate (`executor/`)
**Purpose**: Sequential execution of plans with tool invocation.

**Key Types**:
- `Executor` - Stateful executor with tool registry and memory
- `ExecutionResult` - Outcome with success status and final response
- `StepResult` - Individual step execution result

**Key Methods**:
- `execute_plan(plan)` - Run all steps sequentially
- `execute_step(step)` - Run single step
- `handle_tool_call(tool_call)` - Invoke tool with parameters

**Dependencies**: `planner`, `tools`, `memory`, `core`

**When to use**: Execute validated plans after guardrail checks.

---

### Guardrails Crate (`guardrails/`)
**Purpose**: Safety validation before plan execution.

**Key Trait**:
- `Guardrail` - Trait with `validate(plan) -> Result<()>`

**Registry**:
- `GuardrailRegistry` - Collection of active guardrails

**Built-in Guardrails**:
- `FilePathGuardrail` - Restrict file operations to allowed directories
- `RateLimitGuardrail` - Enforce API call limits per minute

**Dependencies**: `planner`, `core`

**When to use**: Validate plans before execution to prevent unauthorized actions.

---

### Rules Crate (`rules/`)
**Purpose**: Customize agent behavior through prompt modification.

**Key Trait**:
- `Rule` - Trait with `apply(context)` and `priority()`

**Engine**:
- `RuleEngine` - Ordered collection of rules

**Built-in Rules**:
- `ResponseLengthRule` - Limit response word count
- `ToneRule` - Guide response style (Formal, Casual, Technical)

**Dependencies**: `core`

**When to use**: Apply rules before planning to modify LLM behavior.

---

### CLI Crate (`cli/`)
**Purpose**: Command-line interface for agent interaction.

**Modes**:
- **REPL Mode** - Interactive conversation with history
- **Single-Turn Mode** - One query, one response

**Features**:
- Colored output (errors in red, success in green)
- Line editing with rustyline
- Conversation history display
- Verbose logging option

**Dependencies**: `clap`, `rustyline`, `colored`, all framework crates

**When to use**: Run as binary for testing and demonstration.

---

### Examples Crate (`examples/`)
**Purpose**: Demonstrate framework usage patterns.

**Examples**:
1. **chatbot.rs** - Basic conversation (LLM + memory only)
2. **research.rs** - Tool-enabled agent (web search, file reading)
3. **file_manager.rs** - Guardrail demonstration (safe file operations)

**Configuration**: Each example has a corresponding YAML file in `examples/configs/`

**When to use**: Study examples to learn framework patterns and best practices.

## Development

### Building

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --release --workspace
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p core

# Run integration tests
cargo test --test agent_flow
```

### Documentation

```bash
# Generate and open documentation
cargo doc --open --no-deps

# Check documentation
cargo doc --workspace --no-deps
```

### Code Quality

```bash
# Run clippy for linting
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Adding new LLM providers
- Creating custom tools
- Implementing guardrails and rules
- Code style and testing requirements

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Extending the Framework

### Adding a New LLM Provider

1. Create a new module in `llm/src/your_provider/`
2. Define request/response types matching the provider's API
3. Implement the `LLMProvider` trait
4. Add to the factory in `llm/src/factory.rs`

Example:
```rust
// llm/src/my_provider/mod.rs
use async_trait::async_trait;
use crate::provider::LLMProvider;

pub struct MyProvider {
    api_key: String,
    client: ApiClient,
}

#[async_trait]
impl LLMProvider for MyProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        // Implementation
    }
}
```

### Creating a Custom Tool

1. Create a struct for your tool
2. Implement the `Tool` trait
3. Define parameter schema using JSON Schema
4. Register with `ToolRegistry`

Example:
```rust
use async_trait::async_trait;
use tools::Tool;

pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str { "weather" }
    
    fn description(&self) -> &str {
        "Get current weather for a location"
    }
    
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            },
            "required": ["location"]
        })
    }
    
    async fn execute(&self, params: Value) -> Result<Value> {
        let location = params["location"].as_str().unwrap();
        // Call weather API
        Ok(json!({"temperature": 72, "condition": "sunny"}))
    }
}
```

### Implementing a Custom Guardrail

1. Create a struct for your guardrail
2. Implement the `Guardrail` trait
3. Add validation logic in the `validate` method
4. Register with `GuardrailRegistry`

Example:
```rust
use guardrails::Guardrail;

pub struct TokenLimitGuardrail {
    max_tokens: usize,
}

impl Guardrail for TokenLimitGuardrail {
    fn name(&self) -> &str { "token_limit" }
    
    fn validate(&self, plan: &Plan) -> Result<()> {
        let total_tokens = estimate_plan_tokens(plan);
        if total_tokens > self.max_tokens {
            return Err(AgentError::GuardrailViolation(
                format!("Plan exceeds token limit: {} > {}", 
                    total_tokens, self.max_tokens)
            ));
        }
        Ok(())
    }
}
```

## Blog Series

This framework was developed as part of a comprehensive blog series on building AI agents in Rust. Each day of development corresponds to a blog post that explains the concepts, design decisions, and implementation details.

**Blog Series**: [Building AI Agents in Rust - A 14-Day Journey](#)

Topics covered:
- Day 1-3: Foundation (workspace setup, configuration, HTTP communication)
- Day 4-7: Capabilities (LLM integration, memory, tools)
- Day 8-11: Intelligence (planning, execution, guardrails, rules)
- Day 12-14: Interface (CLI, examples, testing, documentation)

## Resources

### Rust Learning
- [The Rust Book](https://doc.rust-lang.org/book/) - Essential Rust fundamentals
- [Async Rust Book](https://rust-lang.github.io/async-book/) - Understanding async/await
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Best practices for API design
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) - Common patterns used in this framework

### AI Agent Architecture
- [ReAct Paper](https://arxiv.org/abs/2210.03629) - Reasoning and Acting in Language Models (used in planner)
- [Chain-of-Thought Paper](https://arxiv.org/abs/2201.11903) - Prompting strategies for better reasoning
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling) - Tool integration patterns
- [Anthropic Claude Docs](https://docs.anthropic.com/claude/docs) - Claude API reference

### Crate Documentation
- [thiserror](https://docs.rs/thiserror) - Error handling patterns
- [async-trait](https://docs.rs/async-trait) - Async trait support
- [reqwest](https://docs.rs/reqwest) - HTTP client
- [tiktoken-rs](https://docs.rs/tiktoken-rs) - Token counting
- [clap](https://docs.rs/clap) - CLI argument parsing

## Troubleshooting

### API Key Issues
```bash
# Verify your API key is set
echo $OPENAI_API_KEY

# If empty, set it
export OPENAI_API_KEY="sk-..."
```

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build --workspace

# Update dependencies
cargo update
```

### Test Failures
```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name -- --nocapture
```

### Integration Test API Costs
Integration tests that call real LLM APIs are marked with `#[ignore]`. Run them explicitly:
```bash
cargo test --test openai_integration -- --ignored
```

## Performance Considerations

This framework prioritizes clarity and learning over performance. For production use, consider:

- **Async Runtime**: Uses tokio with `full` features; minimize features for smaller binaries
- **Token Counting**: tiktoken-rs is OpenAI-specific; implement provider-specific counting
- **Memory Storage**: InMemoryStore is not persistent; implement file or database backends
- **Error Handling**: Detailed errors are helpful for debugging but may expose sensitive info
- **Retry Logic**: Fixed exponential backoff; consider adaptive strategies for production

## Acknowledgments

This framework was built as an educational resource to demonstrate production-quality AI agent architecture in Rust. It prioritizes clarity and learning over performance optimization.

Special thanks to:
- The Rust community for excellent documentation and libraries
- OpenAI and Anthropic for accessible LLM APIs
- Contributors and learners who provide feedback and improvements
