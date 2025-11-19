use agent_core::{AgentError, Result};
use config::LLMConfig;

use crate::{anthropic::AnthropicProvider, openai::OpenAIProvider, LLMProvider};

/// Create an LLM provider instance from configuration
///
/// # Arguments
/// * `config` - LLM configuration specifying provider type and parameters
///
/// # Returns
/// * `Result<Box<dyn LLMProvider>>` - Provider instance or error
///
/// # Errors
/// Returns an error if:
/// - The provider type is unknown
/// - Provider initialization fails
///
/// # Supported Providers
/// - "openai" - OpenAI GPT models
/// - "anthropic" - Anthropic Claude models
pub fn create_provider(config: &LLMConfig) -> Result<Box<dyn LLMProvider>> {
    match config.provider.as_str() {
        "openai" => {
            let provider = OpenAIProvider::new(config)?;
            Ok(Box::new(provider))
        }
        "anthropic" => {
            let provider = AnthropicProvider::new(config)?;
            Ok(Box::new(provider))
        }
        _ => Err(AgentError::Config(format!(
            "Unknown LLM provider: '{}'. Supported providers: openai, anthropic",
            config.provider
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_openai_provider() {
        let config = LLMConfig {
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
            temperature: 0.7,
            max_tokens: 2000,
        };

        let result = create_provider(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_anthropic_provider() {
        let config = LLMConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-sonnet-20240229".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
            temperature: 0.7,
            max_tokens: 2000,
        };

        let result = create_provider(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_unknown_provider() {
        let config = LLMConfig {
            provider: "unknown".to_string(),
            model: "some-model".to_string(),
            api_key: "test-key".to_string(),
            base_url: None,
            temperature: 0.7,
            max_tokens: 2000,
        };

        let result = create_provider(&config);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert!(err.to_string().contains("Unknown LLM provider"));
            assert!(err.to_string().contains("unknown"));
        }
    }
}
