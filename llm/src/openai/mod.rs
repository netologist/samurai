pub mod types;

use agent_core::{AgentError, Message, Result, Role};
use async_trait::async_trait;
use communication::ApiClient;
use config::LLMConfig;

use crate::LLMProvider;

pub use types::{ChatCompletionRequest, ChatCompletionResponse, OpenAIMessage};

/// OpenAI LLM provider implementation
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    temperature: f32,
    max_tokens: usize,
    client: ApiClient,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider from configuration
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
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            client: ApiClient::new(),
        })
    }

    /// Convert framework Message to OpenAI message format
    fn convert_message(message: &Message) -> types::OpenAIMessage {
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };

        types::OpenAIMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Convert multiple framework messages to OpenAI format
    fn convert_messages(messages: &[Message]) -> Vec<types::OpenAIMessage> {
        messages.iter().map(Self::convert_message).collect()
    }
}


#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        // Convert framework messages to OpenAI format
        let openai_messages = Self::convert_messages(messages);

        // Build the request
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
        };

        // Call OpenAI API
        let url = "https://api.openai.com/v1/chat/completions";
        
        // Create a custom client with authorization header
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .timeout(self.client.timeout())
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AgentError::LLMProvider(format!("OpenAI API request timeout: {}", e))
                } else if e.is_connect() {
                    AgentError::LLMProvider(format!("OpenAI API connection error: {}", e))
                } else if e.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    AgentError::LLMProvider("OpenAI API authentication failed: Invalid API key".to_string())
                } else if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    AgentError::LLMProvider("OpenAI API rate limit exceeded".to_string())
                } else {
                    AgentError::LLMProvider(format!("OpenAI API request failed: {}", e))
                }
            })?;

        // Check for HTTP errors
        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error response".to_string());
            
            return Err(AgentError::LLMProvider(format!(
                "OpenAI API HTTP {} error: {}",
                status, error_text
            )));
        }

        // Deserialize the response
        let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
            AgentError::LLMProvider(format!("Failed to deserialize OpenAI response: {}", e))
        })?;

        // Extract the response text from choices[0].message.content
        completion
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or_else(|| {
                AgentError::LLMProvider("OpenAI response contained no choices".to_string())
            })
    }
}
