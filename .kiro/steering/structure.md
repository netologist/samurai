# Project Structure

## Workspace Organization

The project uses a Cargo workspace with 12 member crates organized into three architectural layers.

## Foundation Layer

**core/** - Fundamental types and error handling
- `Message`, `Role` enums for conversation representation
- `AgentError` with thiserror for structured errors
- `Result<T>` type alias used throughout framework

**config/** - Configuration management
- Load from YAML/TOML files
- Environment variable overrides
- Validation for required fields

**communication/** - HTTP client utilities
- Retry logic with exponential backoff
- Shared HTTP client functionality

## Capability Layer

**llm/** - LLM provider implementations
- `LLMProvider` trait for unified interface
- `anthropic/` and `openai/` submodules with types
- Factory pattern for provider instantiation

**memory/** - Conversation storage
- `MemoryStore` trait for storage backends
- `InMemoryStore` implementation
- Token counting and budget management
- `ConversationHistory` wrapper with convenience methods

**tools/** - Tool system
- `Tool` trait with async execution
- `ToolRegistry` for management
- Implementations: `calculator.rs`, `file_reader.rs`, `web_search.rs`
- JSON Schema for parameter validation

## Intelligence Layer

**planner/** - Task decomposition
- LLM-based plan generation
- `Plan` and `Step` types
- Plan validation against available tools

**executor/** - Plan execution
- Sequential step execution
- Tool invocation with parameter passing
- Result collection and error handling

**guardrails/** - Safety validation
- `Guardrail` trait for custom rules
- `GuardrailRegistry` for multiple guardrails
- Implementations: `file_path.rs`, `rate_limit.rs`

**rules/** - Behavior customization
- `Rule` trait for prompt modification
- Priority-based ordering
- Implementations: `response_length.rs`, `tone.rs`

## Interface Layer

**cli/** - Command-line interface
- `main.rs`: Entry point and argument parsing
- `repl.rs`: Interactive REPL mode
- `single.rs`: Single-turn query mode
- `agent.rs`: Agent orchestration logic

## Supporting Directories

**examples/** - Example agents with configs
- `chatbot.rs`: Basic conversation
- `research.rs`: Tool-enabled agent
- `file_manager.rs`: Guardrail demonstration
- `configs/`: YAML configuration files

**tests/** - Integration tests
- `agent_flow.rs`: End-to-end testing with mocks

**target/** - Build artifacts (gitignored)

## Module Conventions

- Each crate has `src/lib.rs` as entry point
- Public API exported via `pub use` in lib.rs
- Submodules organized by functionality
- Comprehensive doc comments with examples
- Tests colocated with implementation using `#[cfg(test)]`
