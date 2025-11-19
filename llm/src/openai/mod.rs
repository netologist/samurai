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
    base_url: String,
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
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
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

        let url = format!("{}/chat/completions", self.base_url);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let completion: ChatCompletionResponse = self
            .client
            .post_json_with_headers(&url, &request, headers)
            .await?;

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
