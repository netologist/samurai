//! Configuration management for the AI agent framework.
//!
//! This crate provides configuration loading and validation from multiple sources:
//! - YAML configuration files
//! - Environment variables
//! - Programmatic configuration
//!
//! # Example
//!
//! ```no_run
//! use config::{load_from_file, validate};
//! use std::path::Path;
//!
//! let config = load_from_file(Path::new("config.yaml")).unwrap();
//! validate(&config).unwrap();
//! ```

use agent_core::{AgentError, Result};
use serde::Deserialize;
use std::path::Path;

/// Top-level configuration structure for the AI agent framework
#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    /// LLM provider configuration
    pub llm: LLMConfig,
    /// Memory system configuration
    pub memory: MemoryConfig,
    /// List of enabled tools
    #[serde(default)]
    pub tools: Vec<String>,
    /// List of enabled guardrails
    #[serde(default)]
    pub guardrails: Vec<String>,
}

/// Configuration for LLM providers (OpenAI, Anthropic, etc.)
#[derive(Debug, Clone, Deserialize)]
pub struct LLMConfig {
    /// Provider name (e.g., "openai", "anthropic")
    pub provider: String,
    /// Model name (e.g., "gpt-4", "claude-3-sonnet-20240229")
    pub model: String,
    /// API key for authentication
    pub api_key: String,
    /// Base URL for the API
    #[serde(default)]
    pub base_url: Option<String>,
    /// Temperature for response generation (0.0 to 2.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
}

/// Configuration for the memory system
#[derive(Debug, Clone, Deserialize)]
pub struct MemoryConfig {
    /// Maximum number of messages to retain
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
    /// Token budget for context window
    #[serde(default = "default_token_budget")]
    pub token_budget: usize,
}

// Default value functions for serde
fn default_temperature() -> f32 {
    0.7
}

fn default_max_tokens() -> usize {
    2000
}

fn default_max_messages() -> usize {
    50
}

fn default_token_budget() -> usize {
    4000
}

/// Load agent configuration from a YAML file
///
/// # Arguments
/// * `path` - Path to the YAML configuration file
///
/// # Returns
/// * `Result<AgentConfig>` - Parsed configuration or error
///
/// # Errors
/// Returns an error if:
/// - The file cannot be read
/// - The YAML is malformed
/// - Required fields are missing
pub fn load_from_file(path: &Path) -> Result<AgentConfig> {
    let contents = std::fs::read_to_string(path).map_err(|e| {
        AgentError::Config(format!(
            "Failed to read config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    let config: AgentConfig = serde_yaml::from_str(&contents).map_err(|e| {
        AgentError::Config(format!(
            "Failed to parse config file '{}': {}",
            path.display(),
            e
        ))
    })?;

    Ok(config)
}

/// Load agent configuration from standard locations
///
/// Search order:
/// 1. `AGENT_CONFIG_PATH` environment variable
/// 2. Current directory: `agent_config.yaml`
/// 3. User config directory: `~/.config/ai-agent/config.yaml`
/// 4. Fallback to environment variables
pub fn load_defaults() -> Result<AgentConfig> {
    // 1. Check AGENT_CONFIG_PATH
    if let Ok(path) = std::env::var("AGENT_CONFIG_PATH") {
        let path = Path::new(&path);
        if path.exists() {
            return load_from_file(path);
        }
    }

    // 2. Check current directory
    let current_dir_config = Path::new("agent_config.yaml");
    if current_dir_config.exists() {
        return load_from_file(current_dir_config);
    }

    // 3. Check user config directory
    if let Some(config_dir) = dirs::config_dir() {
        let user_config = config_dir.join("ai-agent").join("config.yaml");
        if user_config.exists() {
            return load_from_file(&user_config);
        }
    }

    // 4. Fallback to environment variables
    from_env()
}

/// Merge two configurations, with environment config taking precedence
///
/// # Arguments
/// * `file_config` - Configuration loaded from file
/// * `env_config` - Configuration loaded from environment variables
///
/// # Returns
/// * `AgentConfig` - Merged configuration with env values overriding file values
///
/// Environment variables override file-based configuration for:
/// - LLM provider, model, API key, temperature, and max_tokens
/// - Memory settings are taken from file config if present
/// - Tools and guardrails are taken from file config
pub fn merge(mut file_config: AgentConfig, env_config: AgentConfig) -> AgentConfig {
    // Override LLM config with env values
    file_config.llm = env_config.llm;

    // Keep file-based memory, tools, and guardrails settings
    // (env config doesn't provide these)

    file_config
}

/// Validate that required configuration fields are present
///
/// # Arguments
/// * `config` - Configuration to validate
///
/// # Returns
/// * `Result<()>` - Ok if valid, error otherwise
///
/// # Errors
/// Returns an error if:
/// - API key is empty
/// - Provider is empty
/// - Model is empty
pub fn validate(config: &AgentConfig) -> Result<()> {
    if config.llm.api_key.is_empty() {
        return Err(AgentError::Config(
            "API key is required but not provided".to_string(),
        ));
    }

    if config.llm.provider.is_empty() {
        return Err(AgentError::Config(
            "LLM provider is required but not provided".to_string(),
        ));
    }

    if config.llm.model.is_empty() {
        return Err(AgentError::Config(
            "Model name is required but not provided".to_string(),
        ));
    }

    if config.llm.temperature < 0.0 || config.llm.temperature > 2.0 {
        return Err(AgentError::Config(format!(
            "Temperature must be between 0.0 and 2.0, got {}",
            config.llm.temperature
        )));
    }

    if config.llm.max_tokens == 0 {
        return Err(AgentError::Config(
            "Max tokens must be greater than 0".to_string(),
        ));
    }

    if config.memory.max_messages == 0 {
        return Err(AgentError::Config(
            "Max messages must be greater than 0".to_string(),
        ));
    }

    if config.memory.token_budget == 0 {
        return Err(AgentError::Config(
            "Token budget must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

/// Load agent configuration from environment variables
///
/// Reads the following environment variables:
/// - `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` - API key for authentication
/// - `LLM_PROVIDER` - Provider name (defaults to "openai")
/// - `MODEL` - Model name (defaults to "gpt-3.5-turbo")
/// - `TEMPERATURE` - Temperature setting (defaults to 0.7)
/// - `MAX_TOKENS` - Maximum tokens (defaults to 2000)
///
/// # Returns
/// * `Result<AgentConfig>` - Configuration built from environment variables
///
/// # Errors
/// Returns an error if required environment variables are missing
pub fn from_env() -> Result<AgentConfig> {
    let provider = std::env::var("LLM_PROVIDER").unwrap_or_else(|_| "openai".to_string());

    let api_key = match provider.as_str() {
        "openai" => std::env::var("OPENAI_API_KEY").map_err(|_| {
            AgentError::Config("OPENAI_API_KEY environment variable not set".to_string())
        })?,
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AgentError::Config("ANTHROPIC_API_KEY environment variable not set".to_string())
        })?,
        _ => {
            return Err(AgentError::Config(format!(
                "Unknown provider '{}'. Set OPENAI_API_KEY or ANTHROPIC_API_KEY",
                provider
            )));
        }
    };

    let model = std::env::var("MODEL").unwrap_or_else(|_| match provider.as_str() {
        "openai" => "gpt-3.5-turbo".to_string(),
        "anthropic" => "claude-3-sonnet-20240229".to_string(),
        _ => "gpt-3.5-turbo".to_string(),
    });

    let temperature = std::env::var("TEMPERATURE")
        .ok()
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.7);

    let max_tokens = std::env::var("MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(2000);

    Ok(AgentConfig {
        llm: LLMConfig {
            provider: provider.clone(),
            model,
            api_key,
            base_url: Some(match provider.as_str() {
                "openai" => "https://api.openai.com/v1".to_string(),
                "anthropic" => "https://api.anthropic.com/v1".to_string(),
                _ => "".to_string(),
            }),
            temperature,
            max_tokens,
        },
        memory: MemoryConfig {
            max_messages: default_max_messages(),
            token_budget: default_token_budget(),
        },
        tools: Vec::new(),
        guardrails: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::io::Write;

    #[test]
    fn test_config_defaults() {
        let config_str = r#"
            llm:
              provider: openai
              model: gpt-4
              api_key: test-key
            memory: {}
        "#;

        let config: AgentConfig = serde_yaml::from_str(config_str).unwrap();
        assert_eq!(config.llm.temperature, 0.7);
        assert_eq!(config.llm.max_tokens, 2000);
        assert_eq!(config.memory.max_messages, 50);
        assert_eq!(config.memory.token_budget, 4000);
    }

    #[test]
    fn test_load_from_file() {
        let config_content = r#"
llm:
  provider: openai
  model: gpt-4
  api_key: test-key-123
  temperature: 0.8
  max_tokens: 1500
memory:
  max_messages: 30
  token_budget: 3000
tools:
  - calculator
  - file_reader
guardrails:
  - file_path
"#;

        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_config.yaml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let config = load_from_file(&config_path).unwrap();

        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.llm.api_key, "test-key-123");
        assert_eq!(config.llm.temperature, 0.8);
        assert_eq!(config.llm.max_tokens, 1500);
        assert_eq!(config.memory.max_messages, 30);
        assert_eq!(config.memory.token_budget, 3000);
        assert_eq!(config.tools, vec!["calculator", "file_reader"]);
        assert_eq!(config.guardrails, vec!["file_path"]);

        std::fs::remove_file(config_path).unwrap();
    }

    #[test]
    fn test_load_from_file_missing() {
        let result = load_from_file(Path::new("/nonexistent/config.yaml"));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to read config file")
        );
    }

    #[test]
    fn test_load_from_file_invalid_yaml() {
        let config_content = "invalid: yaml: content: [[[";

        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_invalid_config.yaml");
        let mut file = std::fs::File::create(&config_path).unwrap();
        file.write_all(config_content.as_bytes()).unwrap();

        let result = load_from_file(&config_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse config file")
        );

        std::fs::remove_file(config_path).unwrap();
    }

    #[test]
    #[serial]
    fn test_from_env_openai() {
        // Save existing env vars
        let saved_provider = std::env::var("LLM_PROVIDER").ok();
        let saved_openai_key = std::env::var("OPENAI_API_KEY").ok();
        let saved_anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_model = std::env::var("MODEL").ok();
        let saved_temp = std::env::var("TEMPERATURE").ok();
        let saved_tokens = std::env::var("MAX_TOKENS").ok();

        unsafe {
            std::env::set_var("LLM_PROVIDER", "openai");
            std::env::set_var("OPENAI_API_KEY", "test-openai-key");
            std::env::set_var("MODEL", "gpt-4");
            std::env::set_var("TEMPERATURE", "0.9");
            std::env::set_var("MAX_TOKENS", "1000");
        }

        let config = from_env().unwrap();

        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.api_key, "test-openai-key");
        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.llm.temperature, 0.9);
        assert_eq!(config.llm.max_tokens, 1000);

        // Restore env vars
        unsafe {
            if let Some(v) = saved_provider {
                std::env::set_var("LLM_PROVIDER", v);
            } else {
                std::env::remove_var("LLM_PROVIDER");
            }
            if let Some(v) = saved_openai_key {
                std::env::set_var("OPENAI_API_KEY", v);
            } else {
                std::env::remove_var("OPENAI_API_KEY");
            }
            if let Some(v) = saved_anthropic_key {
                std::env::set_var("ANTHROPIC_API_KEY", v);
            } else {
                std::env::remove_var("ANTHROPIC_API_KEY");
            }
            if let Some(v) = saved_model {
                std::env::set_var("MODEL", v);
            } else {
                std::env::remove_var("MODEL");
            }
            if let Some(v) = saved_temp {
                std::env::set_var("TEMPERATURE", v);
            } else {
                std::env::remove_var("TEMPERATURE");
            }
            if let Some(v) = saved_tokens {
                std::env::set_var("MAX_TOKENS", v);
            } else {
                std::env::remove_var("MAX_TOKENS");
            }
        }
    }

    #[test]
    fn test_from_env_anthropic() {
        // Save existing env vars
        let saved_provider = std::env::var("LLM_PROVIDER").ok();
        let saved_openai_key = std::env::var("OPENAI_API_KEY").ok();
        let saved_anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_model = std::env::var("MODEL").ok();
        let saved_temp = std::env::var("TEMPERATURE").ok();
        let saved_tokens = std::env::var("MAX_TOKENS").ok();

        unsafe {
            std::env::set_var("LLM_PROVIDER", "anthropic");
            std::env::set_var("ANTHROPIC_API_KEY", "test-anthropic-key");
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("MODEL");
            std::env::remove_var("TEMPERATURE");
            std::env::remove_var("MAX_TOKENS");
        }

        let config = from_env().unwrap();

        assert_eq!(config.llm.provider, "anthropic");
        assert_eq!(config.llm.api_key, "test-anthropic-key");
        assert_eq!(config.llm.model, "claude-3-sonnet-20240229");
        assert_eq!(config.llm.temperature, 0.7);
        assert_eq!(config.llm.max_tokens, 2000);

        // Restore env vars
        unsafe {
            if let Some(v) = saved_provider {
                std::env::set_var("LLM_PROVIDER", v);
            } else {
                std::env::remove_var("LLM_PROVIDER");
            }
            if let Some(v) = saved_openai_key {
                std::env::set_var("OPENAI_API_KEY", v);
            } else {
                std::env::remove_var("OPENAI_API_KEY");
            }
            if let Some(v) = saved_anthropic_key {
                std::env::set_var("ANTHROPIC_API_KEY", v);
            } else {
                std::env::remove_var("ANTHROPIC_API_KEY");
            }
            if let Some(v) = saved_model {
                std::env::set_var("MODEL", v);
            } else {
                std::env::remove_var("MODEL");
            }
            if let Some(v) = saved_temp {
                std::env::set_var("TEMPERATURE", v);
            } else {
                std::env::remove_var("TEMPERATURE");
            }
            if let Some(v) = saved_tokens {
                std::env::set_var("MAX_TOKENS", v);
            } else {
                std::env::remove_var("MAX_TOKENS");
            }
        }
    }

    #[test]
    #[serial]
    fn test_from_env_defaults() {
        // Save existing env vars
        let saved_provider = std::env::var("LLM_PROVIDER").ok();
        let saved_openai_key = std::env::var("OPENAI_API_KEY").ok();
        let saved_anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_model = std::env::var("MODEL").ok();
        let saved_temp = std::env::var("TEMPERATURE").ok();
        let saved_tokens = std::env::var("MAX_TOKENS").ok();

        unsafe {
            std::env::remove_var("LLM_PROVIDER");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::remove_var("MODEL");
            std::env::remove_var("TEMPERATURE");
            std::env::remove_var("MAX_TOKENS");
            std::env::set_var("OPENAI_API_KEY", "test-key");
        }

        let config = from_env().unwrap();

        assert_eq!(config.llm.provider, "openai");
        assert_eq!(config.llm.model, "gpt-3.5-turbo");
        assert_eq!(config.llm.temperature, 0.7);
        assert_eq!(config.llm.max_tokens, 2000);
        assert_eq!(config.memory.max_messages, 50);
        assert_eq!(config.memory.token_budget, 4000);

        // Restore env vars
        unsafe {
            if let Some(v) = saved_provider {
                std::env::set_var("LLM_PROVIDER", v);
            } else {
                std::env::remove_var("LLM_PROVIDER");
            }
            if let Some(v) = saved_openai_key {
                std::env::set_var("OPENAI_API_KEY", v);
            } else {
                std::env::remove_var("OPENAI_API_KEY");
            }
            if let Some(v) = saved_anthropic_key {
                std::env::set_var("ANTHROPIC_API_KEY", v);
            } else {
                std::env::remove_var("ANTHROPIC_API_KEY");
            }
            if let Some(v) = saved_model {
                std::env::set_var("MODEL", v);
            } else {
                std::env::remove_var("MODEL");
            }
            if let Some(v) = saved_temp {
                std::env::set_var("TEMPERATURE", v);
            } else {
                std::env::remove_var("TEMPERATURE");
            }
            if let Some(v) = saved_tokens {
                std::env::set_var("MAX_TOKENS", v);
            } else {
                std::env::remove_var("MAX_TOKENS");
            }
        }
    }

    #[test]
    #[serial]
    fn test_from_env_missing_api_key() {
        // Save existing env vars
        let saved_provider = std::env::var("LLM_PROVIDER").ok();
        let saved_openai_key = std::env::var("OPENAI_API_KEY").ok();
        let saved_anthropic_key = std::env::var("ANTHROPIC_API_KEY").ok();
        let saved_model = std::env::var("MODEL").ok();
        let saved_temp = std::env::var("TEMPERATURE").ok();
        let saved_tokens = std::env::var("MAX_TOKENS").ok();

        unsafe {
            std::env::remove_var("OPENAI_API_KEY");
            std::env::remove_var("ANTHROPIC_API_KEY");
            std::env::set_var("LLM_PROVIDER", "openai");
        }

        let result = from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OPENAI_API_KEY"));

        // Restore env vars
        unsafe {
            if let Some(v) = saved_provider {
                std::env::set_var("LLM_PROVIDER", v);
            } else {
                std::env::remove_var("LLM_PROVIDER");
            }
            if let Some(v) = saved_openai_key {
                std::env::set_var("OPENAI_API_KEY", v);
            } else {
                std::env::remove_var("OPENAI_API_KEY");
            }
            if let Some(v) = saved_anthropic_key {
                std::env::set_var("ANTHROPIC_API_KEY", v);
            } else {
                std::env::remove_var("ANTHROPIC_API_KEY");
            }
            if let Some(v) = saved_model {
                std::env::set_var("MODEL", v);
            } else {
                std::env::remove_var("MODEL");
            }
            if let Some(v) = saved_temp {
                std::env::set_var("TEMPERATURE", v);
            } else {
                std::env::remove_var("TEMPERATURE");
            }
            if let Some(v) = saved_tokens {
                std::env::set_var("MAX_TOKENS", v);
            } else {
                std::env::remove_var("MAX_TOKENS");
            }
        }
    }

    #[test]
    fn test_merge_configs() {
        let file_config = AgentConfig {
            llm: LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                api_key: "file-key".to_string(),
                base_url: None,
                temperature: 0.5,
                max_tokens: 1000,
            },
            memory: MemoryConfig {
                max_messages: 30,
                token_budget: 3000,
            },
            tools: vec!["calculator".to_string()],
            guardrails: vec!["file_path".to_string()],
        };

        let env_config = AgentConfig {
            llm: LLMConfig {
                provider: "anthropic".to_string(),
                model: "claude-3".to_string(),
                api_key: "env-key".to_string(),
                base_url: None,
                temperature: 0.9,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        let merged = merge(file_config, env_config);

        // LLM config should come from env
        assert_eq!(merged.llm.provider, "anthropic");
        assert_eq!(merged.llm.model, "claude-3");
        assert_eq!(merged.llm.api_key, "env-key");
        assert_eq!(merged.llm.temperature, 0.9);
        assert_eq!(merged.llm.max_tokens, 2000);

        // Memory, tools, and guardrails should come from file
        assert_eq!(merged.memory.max_messages, 30);
        assert_eq!(merged.memory.token_budget, 3000);
        assert_eq!(merged.tools, vec!["calculator"]);
        assert_eq!(merged.guardrails, vec!["file_path"]);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = AgentConfig {
            llm: LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                temperature: 0.7,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_validate_empty_api_key() {
        let config = AgentConfig {
            llm: LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: "".to_string(),
                base_url: None,
                temperature: 0.7,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        let result = validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key"));
    }

    #[test]
    fn test_validate_empty_provider() {
        let config = AgentConfig {
            llm: LLMConfig {
                provider: "".to_string(),
                model: "gpt-4".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                temperature: 0.7,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        let result = validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("provider"));
    }

    #[test]
    fn test_validate_invalid_temperature() {
        let config = AgentConfig {
            llm: LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                temperature: 3.0,
                max_tokens: 2000,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        let result = validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Temperature"));
    }

    #[test]
    fn test_validate_zero_max_tokens() {
        let config = AgentConfig {
            llm: LLMConfig {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                api_key: "test-key".to_string(),
                base_url: None,
                temperature: 0.7,
                max_tokens: 0,
            },
            memory: MemoryConfig {
                max_messages: 50,
                token_budget: 4000,
            },
            tools: Vec::new(),
            guardrails: Vec::new(),
        };

        let result = validate(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max tokens"));
    }
}
