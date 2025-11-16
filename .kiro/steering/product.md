# Product Overview

This is an educational Rust framework for building autonomous AI agents with memory, planning, tool execution, and safety guardrails.

## Purpose

The framework demonstrates production-quality AI agent architecture in Rust, prioritizing clarity and learning over performance optimization. It serves as a learning resource with modular workspace structure where each component can be understood independently.

## Core Capabilities

- **Conversational AI**: Multi-turn conversations with context management
- **Tool Integration**: Extensible system for external capabilities (file operations, web search, calculations)
- **Planning & Execution**: LLM-based task decomposition and step-by-step execution
- **Safety Guardrails**: Validation layer to prevent unauthorized or dangerous actions
- **Memory Management**: Token-aware conversation storage and retrieval
- **Multi-Provider Support**: Unified interface for OpenAI and Anthropic LLMs

## Architecture Layers

1. **Foundation**: Core types, configuration, HTTP communication
2. **Capability**: LLM providers, memory, tools
3. **Intelligence**: Planning, execution, guardrails, behavior rules

## Target Users

Developers learning to build AI agents in Rust, with focus on understanding architectural patterns and best practices for production systems.
