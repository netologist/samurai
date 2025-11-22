//! LLM Provider Interface
//!
//! This crate provides a unified interface for interacting with different
//! Large Language Model providers (OpenAI, Anthropic, etc.).
//!
//! # Supported Providers
//!
//! - **OpenAI**: GPT-3.5, GPT-4, and other OpenAI models
//! - **Anthropic**: Claude models (Claude 3 Sonnet, Opus, etc.)
//!
//! # Usage
//!
//! Use the `create_provider` factory function to instantiate a provider
//! from configuration:
//!
//! ```no_run
//! use llm::create_provider;
//! use config::LLMConfig;
//!
//! # async fn example() -> agent_core::Result<()> {
//! let config = LLMConfig {
//!     provider: "openai".to_string(),
//!     model: "gpt-4".to_string(),
//!     api_key: "your-api-key".to_string(),
//!     base_url: None,
//!     temperature: 0.7,
//!     max_tokens: 2000,
//! };
//!
//! let provider = create_provider(&config)?;
//! # Ok(())
//! # }
//! ```

pub mod anthropic;
mod factory;
pub mod openai;
mod provider;

pub use anthropic::AnthropicProvider;
pub use factory::create_provider;
pub use openai::OpenAIProvider;
pub use provider::LLMProvider;
