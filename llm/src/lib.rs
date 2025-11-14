//! LLM Provider Interface
//! 
//! This crate provides a unified interface for interacting with different
//! Large Language Model providers (OpenAI, Anthropic, etc.).

mod provider;
pub mod openai;

pub use openai::OpenAIProvider;
pub use provider::LLMProvider;
