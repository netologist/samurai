use agent_core::{Message, Result};
use async_trait::async_trait;

/// Trait for LLM provider implementations
/// 
/// This trait defines the interface for interacting with different LLM providers
/// (OpenAI, Anthropic, etc.) in a unified way.
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a sequence of messages to the LLM and receive a response
    /// 
    /// # Arguments
    /// * `messages` - A slice of messages representing the conversation history
    /// 
    /// # Returns
    /// * `Result<String>` - The LLM's response text or an error
    async fn send_message(&self, messages: &[Message]) -> Result<String>;
}
