# Ollama Integration Guide

This guide explains how to use the AI Agent Framework with Ollama for running local open-source LLM models.

## Why Ollama?

- **No API Keys**: Run completely offline
- **No Costs**: Free to use, no per-token charges
- **Privacy**: Your data stays on your machine
- **Fast**: No network latency
- **Variety**: Access to many open-source models

## Quick Start

### Using Docker (Recommended)

The easiest way to get started is using our automated setup:

```bash
# One-command setup (starts container and pulls llama2)
make ollama-setup

# Run the chatbot
make run-ollama
```

That's it! The chatbot will start and you can begin conversing immediately.

### Manual Docker Setup

If you prefer step-by-step control:

```bash
# Start the Ollama container
make ollama-start

# Pull a model (default: llama2)
docker exec athena-ollama ollama pull llama2

# Run the example
cargo run --example ollama_chatbot
```

### Native Installation

If you prefer to install Ollama directly on your system:

1. **Download Ollama**: Visit https://ollama.ai and follow installation instructions for your OS
2. **Pull a model**:
   ```bash
   ollama pull llama2
   ```
3. **Run the example**:
   ```bash
   cargo run --example ollama_chatbot
   ```

## Available Models

You can use any model from the [Ollama Library](https://ollama.ai/library). Popular choices:

### General Purpose
- **llama2** (7B) - Meta's Llama 2, balanced performance
- **mistral** (7B) - Fast and capable, great quality
- **phi** (2.7B) - Microsoft's small but powerful model
- **gemma** (2B/7B) - Google's efficient model

### Code-Specialized
- **codellama** (7B) - Optimized for code generation
- **deepseek-coder** (6.7B) - Strong coding capabilities

### Lightweight
- **phi** (2.7B) - Great for lower-end hardware
- **tinyllama** (1.1B) - Ultra-fast for simple tasks

### Switching Models

To use a different model:

```bash
# Pull the model
docker exec athena-ollama ollama pull mistral

# Update the config
# Edit examples/configs/ollama.yaml:
# model: mistral
```

Or set it in code:

```rust
let mut config = config::load_from_file("examples/configs/ollama.yaml")?;
config.llm.model = "mistral".to_string();
```

## Configuration

The Ollama provider is configured in `examples/configs/ollama.yaml`:

```yaml
llm:
  provider: ollama
  model: llama2
  base_url: http://localhost:11434
  temperature: 0.7
  max_tokens: 2000
```

### Configuration Options

- **provider**: Must be `ollama`
- **model**: Any model name from Ollama library
- **base_url**: URL where Ollama is running (default: `http://localhost:11434`)
- **temperature**: Response randomness (0.0 = deterministic, 1.0 = creative)
- **max_tokens**: Maximum response length

## Docker Management

### Makefile Commands

```bash
# Setup (first time only)
make ollama-setup          # Start container + pull model

# Daily usage
make ollama-start          # Start the container
make ollama-stop           # Stop the container
make ollama-logs           # View container logs
make ollama-models         # List downloaded models
make run-ollama            # Run the chatbot example

# Cleanup
docker compose down -v     # Remove container and data
```

### Manual Docker Commands

```bash
# Start container
docker compose up -d

# Pull additional models
docker exec athena-ollama ollama pull mistral
docker exec athena-ollama ollama pull codellama

# List models
docker exec athena-ollama ollama list

# Remove a model
docker exec athena-ollama ollama rm llama2

# View logs
docker compose logs -f ollama
```

## Troubleshooting

### Container Won't Start

**Check if port 11434 is in use**:
```bash
lsof -i :11434
```

**Solution**: Edit `docker-compose.yml` and change the port:
```yaml
ports:
  - "11435:11434"  # Use different host port
```

Then update `examples/configs/ollama.yaml`:
```yaml
base_url: http://localhost:11435
```

### Model Pull Fails

**Check internet connection** and retry:
```bash
docker exec athena-ollama ollama pull llama2
```

**If repeatedly fails**, try a smaller model:
```bash
docker exec athena-ollama ollama pull tinyllama
```

### Slow Responses

**Try a smaller model**:
- `phi` (2.7B) - Faster, still capable
- `tinyllama` (1.1B) - Very fast

**Increase Docker resources**:
- Open Docker Desktop
- Settings → Resources
- Increase CPU and Memory allocation

### Connection Refused

**Check if Ollama is running**:
```bash
curl http://localhost:11434/api/tags
```

**Start the container**:
```bash
make ollama-start
```

**Check health**:
```bash
docker compose ps
```

## Integration in Your Code

### Basic Usage

```rust
use llm::{LLMProvider, OllamaProvider};
use config::LLMConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let config = LLMConfig {
        provider: "ollama".to_string(),
        model: "llama2".to_string(),
        base_url: Some("http://localhost:11434".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(2000),
        ..Default::default()
    };

    let provider = OllamaProvider::new(config)?;
    let response = provider.send_message("Hello!").await?;
    
    println!("{}", response.content);
    Ok(())
}
```

### With Memory

```rust
use llm::{LLMProvider, OllamaProvider};
use memory::{InMemoryStore, MemoryStore};
use core::{Message, Role};

#[tokio::main]
async fn main() -> Result<()> {
    let provider = OllamaProvider::new(config)?;
    let mut memory = InMemoryStore::new();

    // Add user message
    memory.add_message(Message::new(
        Role::User,
        "What's the capital of France?".to_string()
    ))?;

    // Get conversation history
    let history = memory.get_messages()?;
    
    // Send with context
    let response = provider.send_message_with_history(
        "And what's its population?",
        &history
    ).await?;

    println!("{}", response.content);
    Ok(())
}
```

### Factory Pattern

```rust
use llm::create_provider;
use config::load_from_file;

#[tokio::main]
async fn main() -> Result<()> {
    // Load config (automatically detects provider)
    let config = load_from_file("examples/configs/ollama.yaml")?;
    
    // Create provider (works for OpenAI, Anthropic, or Ollama)
    let provider = create_provider(&config.llm)?;
    
    // Use it
    let response = provider.send_message("Hello!").await?;
    println!("{}", response.content);
    Ok(())
}
```

## Performance Comparison

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| llama2 | 7B | Medium | High | General purpose |
| mistral | 7B | Fast | High | Most tasks |
| phi | 2.7B | Fast | Good | Resource-constrained |
| codellama | 7B | Medium | High | Code generation |
| gemma | 2B/7B | Fast | Good | Efficient inference |
| tinyllama | 1.1B | Very Fast | Basic | Simple tasks |

## Resource Requirements

### Minimum
- **CPU**: 4 cores
- **RAM**: 8GB
- **Disk**: 4GB per model
- **Model**: tinyllama (1.1B)

### Recommended
- **CPU**: 8+ cores
- **RAM**: 16GB
- **Disk**: 10GB+ (for multiple models)
- **Model**: llama2, mistral (7B)

### Optimal
- **CPU**: 16+ cores or GPU
- **RAM**: 32GB+
- **Disk**: 50GB+ (for large models)
- **GPU**: NVIDIA with 8GB+ VRAM
- **Model**: Any, including 13B+ models

## Next Steps

1. **Explore the example**: `cargo run --example ollama_chatbot`
2. **Try different models**: Change model in config and restart
3. **Build your agent**: Use the framework to create custom agents
4. **Add tools**: Integrate with `tools` crate for web search, file ops, etc.
5. **Deploy**: Package your agent as a standalone application

## Resources

- [Ollama Official Site](https://ollama.ai)
- [Ollama Model Library](https://ollama.ai/library)
- [Ollama GitHub](https://github.com/ollama/ollama)
- [Framework Examples](./examples/)
- [Framework Documentation](./README.md)

## FAQ

**Q: Can I use Ollama with the other examples (research, file_manager)?**

A: Yes! Just update their config files to use `provider: ollama` and ensure Ollama is running.

**Q: How much disk space do I need?**

A: Models range from 1GB (tinyllama) to 40GB (llama2-70b). Budget 5-10GB for typical usage.

**Q: Can I use GPU acceleration?**

A: Yes, Ollama automatically uses GPU if available. See [Ollama GPU guide](https://github.com/ollama/ollama/blob/main/docs/gpu.md).

**Q: Can I run multiple models simultaneously?**

A: Yes, but each model loads into memory. Ensure you have enough RAM.

**Q: How do I update Ollama?**

A: `docker compose pull ollama` (Docker) or reinstall from ollama.ai (native).

**Q: Is my data sent anywhere?**

A: No, everything runs locally. No data leaves your machine.
