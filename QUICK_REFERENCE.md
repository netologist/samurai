# Quick Reference - AI Agent Framework

## Quick Start Commands

### Option 1: Ollama (Local, No API Key)
```bash
make ollama-setup    # First time: starts container + pulls model
make run-ollama      # Start chatting
```

### Option 2: OpenAI/Anthropic (Cloud)
```bash
export OPENAI_API_KEY="sk-..."        # or ANTHROPIC_API_KEY
cargo run --example chatbot           # Start chatting
```

## Common Commands

### Building & Testing
```bash
cargo build --workspace               # Build all crates
cargo test --workspace                # Run all tests (258 tests)
cargo clippy --workspace              # Lint code
cargo fmt --all                       # Format code
cargo doc --open --no-deps            # Generate docs
```

### Running Examples
```bash
cargo run --example ollama_chatbot    # Local Ollama chatbot
cargo run --example chatbot           # Cloud LLM chatbot
cargo run --example research          # Research assistant
cargo run --example file_manager      # File operations agent
```

### Ollama Management
```bash
make ollama-setup     # First-time setup (container + model)
make ollama-start     # Start Ollama container
make ollama-stop      # Stop Ollama container
make ollama-logs      # View container logs
make ollama-models    # List downloaded models
```

### Manual Ollama Commands
```bash
# Pull models
docker exec samurai-ollama ollama pull llama2
docker exec samurai-ollama ollama pull mistral
docker exec samurai-ollama ollama pull codellama

# List models
docker exec samurai-ollama ollama list

# Remove a model
docker exec samurai-ollama ollama rm llama2

# Health check
curl http://localhost:11434/api/tags
```

## Configuration Files

### Ollama Config
`examples/configs/ollama.yaml`:
```yaml
llm:
  provider: ollama
  model: llama2
  base_url: http://localhost:11434
  temperature: 0.7
  max_tokens: 2000
```

### OpenAI Config
`examples/configs/chatbot.yaml`:
```yaml
llm:
  provider: openai
  model: gpt-4
  api_key: ${OPENAI_API_KEY}
  temperature: 0.7
  max_tokens: 1000
```

### Anthropic Config
`examples/configs/anthropic.yaml`:
```yaml
llm:
  provider: anthropic
  model: claude-3-sonnet-20240229
  api_key: ${ANTHROPIC_API_KEY}
  temperature: 0.7
  max_tokens: 1000
```

## Quick Code Snippets

### Basic Agent
```rust
use llm::create_provider;
use config::load_from_file;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_from_file("config.yaml")?;
    let provider = create_provider(&config.llm)?;
    let response = provider.send_message("Hello!").await?;
    println!("{}", response.content);
    Ok(())
}
```

### Agent with Memory
```rust
use llm::create_provider;
use memory::{InMemoryStore, MemoryStore};
use core::{Message, Role};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_from_file("config.yaml")?;
    let provider = create_provider(&config.llm)?;
    let mut memory = InMemoryStore::new();
    
    memory.add_message(Message::new(Role::User, "Hi!".to_string()))?;
    let history = memory.get_messages()?;
    
    let response = provider.send_message_with_history("Hi!", &history).await?;
    println!("{}", response.content);
    Ok(())
}
```

## Available Models

### Ollama (Local)
| Model | Size | Best For |
|-------|------|----------|
| llama2 | 7B | General purpose |
| mistral | 7B | Fast & capable |
| phi | 2.7B | Resource-constrained |
| codellama | 7B | Code generation |
| gemma | 2B/7B | Efficient inference |
| tinyllama | 1.1B | Simple tasks |

### OpenAI (Cloud)
- `gpt-4` - Most capable
- `gpt-4-turbo` - Fast + large context
- `gpt-3.5-turbo` - Cost-effective

### Anthropic (Cloud)
- `claude-3-opus-20240229` - Most capable
- `claude-3-sonnet-20240229` - Balanced
- `claude-3-haiku-20240307` - Fast

## Troubleshooting

### Ollama Not Responding
```bash
make ollama-start              # Start container
docker compose logs ollama     # Check logs
curl http://localhost:11434/api/tags  # Test endpoint
```

### Port Already in Use
Edit `docker-compose.yml`:
```yaml
ports:
  - "11435:11434"  # Change host port
```

### Model Pull Fails
```bash
# Check internet connection
# Try smaller model first
docker exec samurai-ollama ollama pull tinyllama
```

### Tests Failing
```bash
cargo clean                    # Clean build artifacts
cargo build --workspace        # Rebuild
cargo test --workspace         # Run tests
```

## Project Structure

```
ai-agent-framework/
├── core/           # Basic types (Message, Role, Error)
├── config/         # Configuration management
├── llm/            # LLM providers (OpenAI, Anthropic, Ollama)
├── memory/         # Conversation storage
├── tools/          # Tool system (Calculator, FileReader, etc.)
├── planner/        # Task decomposition
├── executor/       # Plan execution
├── guardrails/     # Safety validation
├── rules/          # Behavior customization
├── cli/            # Command-line interface
└── examples/       # Example agents
```

## Documentation

- **README.md** - Project overview
- **OLLAMA_SETUP.md** - Complete Ollama guide
- **IMPROVEMENTS.md** - All improvements made
- **scripts/README.md** - Script documentation
- **examples/README.md** - Example descriptions

## Getting Help

1. **Check documentation**: Start with `README.md`
2. **View examples**: See `examples/` directory
3. **Run tests**: `cargo test --workspace`
4. **Generate API docs**: `cargo doc --open --no-deps`
5. **Check logs**: `make ollama-logs` or `docker compose logs`

## Resource Requirements

### Minimum (tinyllama)
- 4 CPU cores
- 8GB RAM
- 4GB disk

### Recommended (llama2, mistral)
- 8+ CPU cores
- 16GB RAM
- 10GB+ disk

### Optimal (any model)
- 16+ cores or GPU
- 32GB+ RAM
- 50GB+ disk

## Next Steps

1. **Try examples**: `make ollama-setup && make run-ollama`
2. **Read docs**: `OLLAMA_SETUP.md` for complete guide
3. **Build custom agent**: Use framework components
4. **Add tools**: Integrate with `tools` crate
5. **Deploy**: Package as standalone app

## Links

- [Ollama](https://ollama.ai) - Local LLM runtime
- [OpenAI](https://platform.openai.com) - Cloud LLM API
- [Anthropic](https://www.anthropic.com) - Claude API
- [Rust](https://www.rust-lang.org) - Programming language

---

**All tests passing**: 258/258 ✅  
**Warnings**: 0 ✅  
**LLM Providers**: 3 (OpenAI, Anthropic, Ollama) ✅  
**Ready for production**: Yes ✅
