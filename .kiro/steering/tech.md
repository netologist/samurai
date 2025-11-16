# Technology Stack

## Build System

- **Cargo Workspace**: Multi-crate workspace with 12 member crates
- **Rust Edition**: 2021
- **Minimum Rust Version**: 1.70 or later

## Core Dependencies

- **serde** (1.0): Serialization/deserialization with derive macros
- **tokio** (1.48): Async runtime with full features
- **thiserror** (1.0): Error type derivation
- **chrono** (0.4): Date/time handling with serde support
- **reqwest** (0.12): HTTP client with JSON support
- **serde_json** (1.0): JSON processing
- **async-trait** (0.1): Async trait support

## CLI Dependencies

- **colored** (2.1): Terminal color output
- **rustyline** (12.0): REPL line editing

## Common Commands

### Building

```bash
# Build all crates
cargo build --workspace

# Build in release mode
cargo build --release --workspace

# Build examples
cargo build --examples
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p core

# Run integration tests
cargo test --test agent_flow
```

### Running Examples

```bash
cargo run --example chatbot
cargo run --example research
cargo run --example file_manager
```

### Code Quality

```bash
# Lint with clippy
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Generate documentation
cargo doc --open --no-deps
```

## API Keys Required

Set environment variables before running:

```bash
export OPENAI_API_KEY="your-key-here"
# or
export ANTHROPIC_API_KEY="your-key-here"
```

## Configuration Format

YAML files in `examples/configs/` with structure:
- `llm`: Provider settings (provider, model, api_key, temperature, max_tokens)
- `memory`: Context management (max_messages, token_budget)
- `tools`: Available tool list
- `guardrails`: Active safety constraints
