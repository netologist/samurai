.PHONY: all build test check fmt clippy clean run-chatbot run-research run-ollama install help
.PHONY: ollama-start ollama-stop ollama-setup ollama-logs ollama-models

# Default target
all: build test

# Build the project
build:
	cargo build --workspace

# Run tests
test:
	cargo test --workspace

# Check code for errors without building
check:
	cargo check --workspace

# Format code
fmt:
	cargo fmt --all

# Run clippy linter
clippy:
	cargo clippy --workspace -- -D warnings

# Run the chatbot example
run-chatbot:
	cargo run --example chatbot

# Run the research example
run-research:
	cargo run --example research

# Run the Ollama chatbot example
run-ollama:
	cargo run --example ollama_chatbot

# Ollama Docker setup and management
ollama-setup:
	@./scripts/setup-ollama.sh

ollama-start:
	@echo "Starting Ollama..."
	@docker compose up -d
	@echo "Ollama is running on http://localhost:11434"

ollama-stop:
	@echo "Stopping Ollama..."
	@docker compose down

ollama-logs:
	@docker compose logs -f ollama

ollama-models:
	@docker exec athena-ollama ollama list

# Clean build artifacts
clean:
	cargo clean

# Install the CLI tool
install:
	cargo install --path cli

# Show help
help:
	@echo "Available targets:"
	@echo ""
	@echo "Build & Test:"
	@echo "  build        - Build the project"
	@echo "  test         - Run tests"
	@echo "  check        - Check code for errors"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy linter"
	@echo "  clean        - Clean build artifacts"
	@echo ""
	@echo "Examples:"
	@echo "  run-chatbot  - Run the chatbot example (requires API key)"
	@echo "  run-research - Run the research example (requires API key)"
	@echo "  run-ollama   - Run the Ollama chatbot (no API key needed)"
	@echo ""
	@echo "Ollama (Local LLM):"
	@echo "  ollama-setup - Setup and pull Ollama model (one-time setup)"
	@echo "  ollama-start - Start Ollama container"
	@echo "  ollama-stop  - Stop Ollama container"
	@echo "  ollama-logs  - View Ollama logs"
	@echo "  ollama-models- List installed models"
	@echo ""
	@echo "Other:"
	@echo "  install      - Install the CLI tool"
	@echo "  help         - Show this help message"
	@echo ""
