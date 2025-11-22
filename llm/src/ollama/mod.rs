pub mod types;

use agent_core::{AgentError, Message, Result, Role};
use async_trait::async_trait;
use communication::ApiClient;
use config::LLMConfig;

use crate::LLMProvider;

pub use types::{ChatRequest, ChatResponse, OllamaMessage};

/// Ollama LLM provider implementation
///
/// Supports running open-source models locally via Ollama.
/// Models include llama2, mistral, codellama, phi, and many others.
///
/// # Example
///
/// ```no_run
/// use llm::OllamaProvider;
/// use config::LLMConfig;
///
/// let config = LLMConfig {
///     provider: "ollama".to_string(),
///     model: "llama2".to_string(),
///     api_key: String::new(), // Not needed for Ollama
///     base_url: Some("http://localhost:11434".to_string()),
///     temperature: 0.7,
///     max_tokens: 2000,
/// };
///
/// let provider = OllamaProvider::new(&config).unwrap();
/// ```
pub struct OllamaProvider {
    model: String,
    base_url: String,
    temperature: Option<f32>,
    max_tokens: Option<usize>,
    client: ApiClient,
}

impl OllamaProvider {
    /// Create a new Ollama provider from configuration
    ///
    /// # Arguments
    /// * `config` - LLM configuration containing model and parameters
    ///
    /// # Returns
    /// * `Result<Self>` - New provider instance or error
    pub fn new(config: &LLMConfig) -> Result<Self> {
        Ok(Self {
            model: config.model.clone(),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            temperature: if config.temperature > 0.0 {
                Some(config.temperature)
            } else {
                None
            },
            max_tokens: if config.max_tokens > 0 {
                Some(config.max_tokens)
            } else {
                None
            },
            client: ApiClient::new(),
        })
    }

    /// Convert framework Message to Ollama message format
    fn convert_message(message: &Message) -> types::OllamaMessage {
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };

        types::OllamaMessage {
            role: role.to_string(),
            content: message.content.clone(),
        }
    }

    /// Convert multiple framework messages to Ollama format
    fn convert_messages(messages: &[Message]) -> Vec<types::OllamaMessage> {
        messages.iter().map(Self::convert_message).collect()
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn send_message(&self, messages: &[Message]) -> Result<String> {
        // Convert framework messages to Ollama format
        let ollama_messages = Self::convert_messages(messages);

        // Build the request
        let request = ChatRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            temperature: self.temperature,
            num_predict: self.max_tokens,
            stream: false,
        };

        let url = format!("{}/api/chat", self.base_url);

        // Ollama doesn't require authentication headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|e| AgentError::LLMProvider(format!("Invalid header value: {}", e)))?,
        );

        let response: ChatResponse = self
            .client
            .post_json_with_headers(&url, &request, headers)
            .await?;

        Ok(response.message.content)
    }
}
