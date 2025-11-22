.PHONY: all build test check fmt clippy clean run-chatbot run-research install help

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

# Clean build artifacts
clean:
	cargo clean

# Install the CLI tool
install:
	cargo install --path cli

# Show help
help:
	@echo "Available targets:"
	@echo "  build        - Build the project"
	@echo "  test         - Run tests"
	@echo "  check        - Check code for errors"
	@echo "  fmt          - Format code"
	@echo "  clippy       - Run clippy linter"
	@echo "  run-chatbot  - Run the chatbot example"
	@echo "  run-research - Run the research example"
	@echo "  clean        - Clean build artifacts"
	@echo "  install      - Install the CLI tool"
	@echo "  help         - Show this help message"
