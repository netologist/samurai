# GEMINI.md

This document provides a comprehensive overview of the AI Agent Framework, a Rust-based framework for building autonomous AI agents.

## Project Overview

The AI Agent Framework is a modular Rust workspace designed to demonstrate the principles of building production-quality AI agents. It provides a clear, well-structured implementation of components for memory, planning, tool execution, and safety guardrails. The framework is intended as an educational resource and a starting point for building custom AI agents.

The project is structured as a Rust workspace with the following crates:

-   `core`: Fundamental types and error handling.
-   `config`: Configuration management from files and environment variables.
-   `communication`: HTTP client utilities.
-   `llm`: A unified interface for multiple LLM providers (OpenAI, Anthropic).
-   `memory`: Conversation storage with token-aware context management.
-   `tools`: An extensible tool system for agent capabilities.
-   `planner`: LLM-based task decomposition using the ReAct pattern.
-   `executor`: Sequential execution of plans with tool invocation.
-   `guardrails`: Safety validation before plan execution.
-   `rules`: Customization of agent behavior through prompt modification.
-   `cli`: A command-line interface for interacting with the agents.
-   `examples`: Example agents demonstrating the framework's capabilities.

## Building and Running

### Prerequisites

-   Rust (version 1.70 or later)
-   An API key for either OpenAI or Anthropic

### Building the Project

To build the entire workspace, run the following command from the root directory:

```bash
cargo build --workspace
```

### Running the Examples

The framework includes several example agents that demonstrate different capabilities.

**1. Simple Chatbot**

This example demonstrates a basic conversational agent with memory but no tools.

```bash
# Set your API key
export OPENAI_API_KEY="sk-..." # or ANTHROPIC_API_KEY

# Run the chatbot example
cargo run --example chatbot
```

**2. Research Assistant**

This example demonstrates an agent with web search and file reading capabilities.

```bash
# Set your API key
export OPENAI_API_KEY="sk-..." # or ANTHROPIC_API_KEY

# Run the research assistant example
cargo run --example research
```

**3. File Manager**

This example demonstrates an agent with file operations and safety guardrails.

```bash
# Set your API key
export OPENAI_API_KEY="sk-..." # or ANTHROPIC_API_KEY

# Run the file manager example
cargo run --example file_manager
```

### Running Tests

To run all tests for the workspace, use the following command:

```bash
cargo test --workspace
```

To run the integration tests, which may make calls to external APIs, use the following command:

```bash
cargo test --test agent_flow -- --ignored
```

## Development Conventions

### Code Style

The project follows the standard Rust formatting conventions. To format the code, run:

```bash
cargo fmt --all
```

### Linting

The project uses Clippy for linting. To check the code for lints, run:

```bash
cargo clippy --workspace -- -D warnings
```

### Documentation

To generate and open the documentation for all crates, run:

```bash
cargo doc --open --no-deps
```
