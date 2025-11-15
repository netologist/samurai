# AI Agent Framework Examples

This directory contains example agents demonstrating different capabilities of the AI agent framework. Each example showcases specific features and use cases, progressing from simple to complex.

## Prerequisites

Before running any examples, you need to:

1. **Set up API keys**: All examples require an LLM provider API key
   ```bash
   # For OpenAI (used in all examples)
   export OPENAI_API_KEY=your_openai_api_key_here
   
   # For Anthropic (if using anthropic provider)
   export ANTHROPIC_API_KEY=your_anthropic_api_key_here
   ```

2. **Build the project**: Ensure all dependencies are compiled
   ```bash
   cargo build --examples
   ```

## Examples Overview

### 1. Simple Chatbot (`chatbot.rs`)

**What it demonstrates:**
- Basic conversational agent with no tools
- LLM integration and memory management
- Interactive REPL for multi-turn conversations
- Configuration loading from YAML files

**Key features:**
- Uses GPT-3.5-turbo for cost-effective conversations
- Maintains conversation context through memory system
- Simple, focused interaction without external capabilities

**Configuration:** `configs/chatbot.yaml`

**Run it:**
```bash
cargo run --example chatbot
```

**Example interactions:**
```
You: Hello! What can you help me with?
Bot: I'm a conversational AI assistant. I can chat with you about various topics...

You: Tell me a joke
Bot: Why did the programmer quit his job? Because he didn't get arrays!

You: exit
Goodbye! Thanks for chatting!
```

**What you'll learn:**
- How to initialize a basic agent with LLM and memory
- How conversation context is maintained across turns
- How to build a simple REPL interface
- Configuration management basics

---

### 2. Research Assistant (`research.rs`)

**What it demonstrates:**
- Agent with tool capabilities (WebSearch, FileReader)
- Multi-step planning and execution
- Tool invocation and result synthesis
- Complex workflow orchestration

**Key features:**
- Uses GPT-4 for advanced reasoning and planning
- WebSearchStub tool (simulates web search)
- FileReader tool for analyzing documents
- Planner decomposes queries into executable steps
- Executor runs steps and invokes tools

**Configuration:** `configs/research.yaml`

**Run it:**
```bash
cargo run --example research
```

**Example interactions:**
```
Research: Search for information about Rust async programming
  → Planning research steps...
  → Plan created with 3 steps
  → Executing research plan...

Research Results:
Based on my research, Rust async programming uses the async/await syntax...
[Detailed response synthesizing information from tools]

Research: Read the README.md file and summarize it
  → Planning research steps...
  → Plan created with 2 steps
  → Executing research plan...

Research Results:
The README describes an AI agent framework built in Rust...
```

**What you'll learn:**
- How tools extend agent capabilities
- How the planner decomposes complex queries
- How the executor orchestrates tool invocations
- Multi-step workflow patterns
- Tool result synthesis

---

### 3. File Manager (`file_manager.rs`)

**What it demonstrates:**
- Safe file operations with guardrails
- FilePathGuardrail for security
- Guardrail validation before execution
- Security-conscious agent design

**Key features:**
- FileReader tool for reading file contents
- FilePathGuardrail restricts access to allowed directories
- Demonstrates guardrail violation handling
- Shows how to build secure agents

**Configuration:** `configs/file_manager.yaml`

**Run it:**
```bash
cargo run --example file_manager
```

**Example interactions:**
```
FileManager: Read the README.md file and summarize it
  → Planning file operations...
  → Plan created with 2 steps
  → Validating with guardrails...
  ✓ Guardrail validation passed
  → Executing file operations...

File Manager Results:
The README file describes an AI agent framework...

FileManager: Read /etc/passwd
  → Planning file operations...
  → Plan created with 2 steps
  → Validating with guardrails...
  ✗ Guardrail violation detected!

Error: Guardrail violation: File path not allowed: /etc/passwd
This file access was blocked by the FilePathGuardrail.
Only files in allowed directories can be accessed.
```

**What you'll learn:**
- How guardrails enforce safety constraints
- How to restrict agent capabilities for security
- Guardrail validation workflow
- Error handling for policy violations
- Security-first agent design patterns

---

## Configuration Files

Each example has a corresponding configuration file in `configs/`:

- **`chatbot.yaml`**: Simple configuration with no tools
- **`research.yaml`**: Configuration with tools enabled and higher token budget
- **`file_manager.yaml`**: Configuration with FileReader tool and FilePathGuardrail

### Configuration Structure

All configuration files follow this structure:

```yaml
# LLM Configuration
llm:
  provider: openai          # or "anthropic"
  model: gpt-3.5-turbo     # or gpt-4, claude-3-sonnet, etc.
  api_key: ${OPENAI_API_KEY}  # Environment variable
  temperature: 0.7          # 0.0 to 2.0
  max_tokens: 500          # Maximum response length

# Memory Configuration
memory:
  max_messages: 20         # Maximum messages to keep
  token_budget: 2000       # Maximum tokens in context

# Tools Configuration
tools:
  - calculator             # Available tools
  - file_reader
  - web_search

# Guardrails Configuration
guardrails:
  - file_path             # Active guardrails
  - rate_limit
```

### Customizing Configurations

You can modify the configuration files to experiment with different settings:

- **Change the model**: Try `gpt-4` for better reasoning or `gpt-3.5-turbo` for speed
- **Adjust temperature**: Lower (0.1-0.3) for focused responses, higher (0.8-1.2) for creativity
- **Modify token limits**: Increase for longer responses, decrease for brevity
- **Enable/disable tools**: Add or remove tools from the list
- **Configure guardrails**: Add safety constraints as needed

---

## Common Commands

### Running Examples

```bash
# Run specific example
cargo run --example chatbot
cargo run --example research
cargo run --example file_manager

# Build all examples
cargo build --examples

# Build in release mode (faster execution)
cargo build --examples --release
cargo run --example chatbot --release
```

### REPL Commands

All examples support these commands in the REPL:

- **`exit`** or **`quit`**: Exit the example
- **`help`**: Show available commands
- **`examples`**: Show example queries (research and file_manager)
- **Ctrl-C**: Interrupt (shows exit message)
- **Ctrl-D**: EOF (exits gracefully)

---

## Troubleshooting

### API Key Not Set

**Error:** `Configuration error: API key not found`

**Solution:** Set the appropriate environment variable:
```bash
export OPENAI_API_KEY=your_key_here
```

### API Rate Limits

**Error:** `LLM provider error: Rate limit exceeded`

**Solution:** 
- Wait a few moments and try again
- Use a different model with higher rate limits
- Implement rate limiting in your configuration

### File Not Found

**Error:** `IO error: No such file or directory`

**Solution:**
- Ensure you're running from the project root directory
- Check that configuration files exist in `examples/configs/`
- Verify file paths in your queries

### Guardrail Violations

**Error:** `Guardrail violation: File path not allowed`

**Solution:**
- This is expected behavior for the file_manager example
- Only files in allowed directories can be accessed
- Modify the allowed paths in the code if needed

---

## Learning Path

We recommend exploring the examples in this order:

1. **Start with Chatbot**: Understand basic agent structure and conversation flow
2. **Move to Research Assistant**: Learn about tools, planning, and execution
3. **Finish with File Manager**: Explore guardrails and security features

Each example builds on concepts from the previous ones, providing a progressive learning experience.

---

## Next Steps

After exploring these examples, you can:

1. **Modify the examples**: Change configurations, add new tools, or customize behavior
2. **Create your own agent**: Use these examples as templates for your use cases
3. **Extend the framework**: Add new tools, guardrails, or LLM providers
4. **Read the documentation**: Explore the main README and crate documentation
5. **Review the source code**: Study the implementation in each crate

---

## Additional Resources

- **Main README**: `../README.md` - Project overview and architecture
- **Design Document**: `.kiro/specs/ai-agent-framework/design.md` - Detailed design
- **Requirements**: `.kiro/specs/ai-agent-framework/requirements.md` - System requirements
- **API Documentation**: Run `cargo doc --open` to view generated docs

---

## Example Output

Here's what you can expect when running the examples:

### Chatbot Output
```
=== Simple Chatbot Example ===
This example demonstrates a basic conversational agent.

Chatbot REPL - Type your messages and press Enter
Commands: 'exit' to quit, 'help' for help

You: Hello!
Bot: Hello! How can I help you today?

You: What's the weather like?
Bot: I don't have access to real-time weather data, but I'd be happy to chat about weather in general...
```

### Research Assistant Output
```
=== Research Assistant Example ===
This example demonstrates an agent with tool capabilities.
Available tools: WebSearch (stub), FileReader

  ✓ Registered WebSearchStub tool
  ✓ Registered FileReader tool

Research Assistant REPL - Ask research questions
The assistant will use tools to gather and analyze information.
Commands: 'exit' to quit, 'help' for help, 'examples' for sample queries

Research: Tell me about Rust
  → Planning research steps...
  → Plan created with 2 steps
  → Executing research plan...

Research Results:
Rust is a systems programming language that focuses on safety, speed, and concurrency...
```

### File Manager Output
```
=== File Manager Example ===
This example demonstrates safe file operations with guardrails.

  ✓ Registered FileReader tool
  ✓ Registered FilePathGuardrail

Allowed file access paths:
  • Current directory: /path/to/project
  • Examples directory: examples/

Attempts to access files outside these paths will be blocked.

File Manager REPL - Ask about files in allowed directories
The agent will use FileReader tool with guardrail protection.
Commands: 'exit' to quit, 'help' for help, 'examples' for sample queries
```

---

## Contributing

If you create interesting examples or improvements:

1. Follow the existing code style and documentation patterns
2. Add comprehensive comments explaining what the example demonstrates
3. Include a configuration file with detailed comments
4. Update this README with your example
5. Test thoroughly before submitting

---

## License

These examples are part of the AI Agent Framework project and follow the same license as the main project.
