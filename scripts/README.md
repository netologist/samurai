# Scripts

This directory contains utility scripts for the AI Agent Framework.

## Ollama Setup Script

The `setup-ollama.sh` script automates the setup of Ollama using Docker for running local LLM models.

### Usage

```bash
# Setup with default model (llama2)
./scripts/setup-ollama.sh

# Setup with a specific model
./scripts/setup-ollama.sh mistral
./scripts/setup-ollama.sh phi
./scripts/setup-ollama.sh codellama
```

### What it does

1. Starts Ollama container using docker-compose
2. Waits for Ollama to be ready
3. Pulls the specified model (default: llama2)
4. Provides helpful commands for next steps

### Available Models

Popular models you can use:
- `llama2` - Meta's Llama 2 (7B) - General purpose
- `mistral` - Mistral 7B - Fast and capable
- `phi` - Microsoft Phi-2 (2.7B) - Small but powerful
- `codellama` - Code Llama (7B) - Specialized for code
- `gemma` - Google Gemma (2B/7B) - Lightweight
- `neural-chat` - Intel's Neural Chat
- `starling-lm` - Starling language model

See [Ollama Library](https://ollama.ai/library) for the full list.

### Requirements

- Docker (with docker-compose)
- Internet connection (for first-time model download)
- ~4GB disk space per model

### Alternative: Using Makefile

You can also use the Makefile commands:

```bash
# One-time setup
make ollama-setup

# Start Ollama
make ollama-start

# Stop Ollama
make ollama-stop

# View logs
make ollama-logs

# List models
make ollama-models
```

### Troubleshooting

**Container won't start:**
```bash
docker compose logs ollama
```

**Model pull fails:**
```bash
# Retry the pull manually
docker exec samurai-ollama ollama pull llama2
```

**Port already in use:**
Edit `docker-compose.yml` and change the port mapping:
```yaml
ports:
  - "11435:11434"  # Use different host port
```

Then update `examples/configs/ollama.yaml` accordingly.

**Running without Docker:**

If you prefer to install Ollama natively:
1. Install from https://ollama.ai
2. Run `ollama pull llama2`
3. Use `cargo run --example ollama_chatbot`
