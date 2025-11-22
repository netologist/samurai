pub mod types;

use agent_core::{AgentError, Message, Result, Role};
use async_trait::async_trait;
use communication::ApiClient;
use config::LLMConfig;

use crate::LLMProvider;

pub use types::{AnthropicMessage, MessagesRequest, MessagesResponse};

/// Anthropic LLM provider implementation
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    temperature: f32,
    max_tokens: usize,
    client: ApiClient,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider from configuration
    ///
    /// # Arguments
    /// * `config` - LLM configuration containing API key, model, and parameters
    ///
    /// # Returns
    /// * `Result<Self>` - New provider instance or error
    pub fn new(config: &LLMConfig) -> Result<Self> {
        Ok(Self {
            api_key: config.api_key.clone(),
            model: config.model.clone(),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string()),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            client: ApiClient::new(),
        })
    }

    /// Convert framework Message to Anthropic message format
    ///
    /// Note: System messages are handled separately and should not be
    /// included in the messages array
    fn convert_message(message: &Message) -> Option<types::AnthropicMessage> {
        match message.role {
            Role::System => None, // System messages go in separate field
            Role::User => Some(types::AnthropicMessage {
                role: "user".to_string(),
                content: message.content.clone(),
            }),
            Role::Assistant => Some(types::AnthropicMessage {
                role: "assistant".to_string(),
                content: message.content.clone(),
            }),
        }
    }

    /// Convert multiple framework messages to Anthropic format
    ///
    /// Separates system messages from user/assistant messages.
    /// Returns (system_message, messages_array)
    fn convert_messages(messages: &[Message]) -> (Option<String>, Vec<types::AnthropicMessage>) {
        let mut system_message: Option<String> = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                Role::System => {
                    // Combine multiple system messages if present
                    if let Some(existing) = system_message.as_mut() {
                        existing.push_str("\n\n");
                        existing.push_str(&message.content);
                    } else {
                        system_message = Some(message.content.clone());
                    }
                }
                _ => {
                    if let Some(anthropic_msg) = Self::convert_message(message) {
                        anthropic_messages.push(anthropic_msg);
                    }
                }
            }
        }

        (system_message, anthropic_messages)
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        // Convert framework messages to Anthropic format, separating system messages
        let (system, anthropic_messages) = Self::convert_messages(messages);

        // Build the request
        let request = MessagesRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
            system,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        let url = format!("{}/messages", self.base_url);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            self.api_key
                .parse()
                .map_err(|e| AgentError::LLMProvider(format!("Invalid API key format: {}", e)))?,
        );
        headers.insert(
            "anthropic-version",
            "2023-06-01"
                .parse()
                .map_err(|e| AgentError::LLMProvider(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|e| AgentError::LLMProvider(format!("Invalid header value: {}", e)))?,
        );

        let messages_response: MessagesResponse = self
            .client
            .post_json_with_headers(&url, &request, headers)
            .await?;

        // Extract the response text from content[0].text
        messages_response
            .content
            .first()
            .map(|content| content.text.clone())
            .ok_or_else(|| {
                AgentError::LLMProvider("Anthropic response contained no content".to_string())
            })
    }
}
